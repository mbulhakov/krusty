use reqwest;
use serde::Deserialize;
use std::borrow::Borrow;
use std::boxed::Box;
use std::env;
use std::error::Error;
use std::fmt;

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct DriveFile {
    name: String,
    id: String,
}
#[derive(Deserialize)]
struct FileListResponse {
    files: Vec<DriveFile>,
}

#[derive(Debug)]
struct DownloadError {
    description: String,
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error occured during download: {}", self.description)
    }
}

impl Error for DownloadError {}

pub async fn download_gachi_from_drive() -> Result<Vec<u8>, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let secret = env::var("GOGLE_CLIENT_SECRET")?;
    let client_id = env::var("GOOGLE_CLIENT_ID")?;
    let refresh_token = env::var("GOOGLE_REFRESH_TOKEN")?;
    let api_key = env::var("GOOGLE_API_KEY")?;
    let gachi_file_name = env::var("GACHI_FILE_NAME")?;

    let token_params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.as_str()),
        ("client_id", client_id.as_str()),
        ("client_secret", secret.as_str()),
        ("scope", "https://www.googleapis.com/auth/drive.readonly"),
    ];

    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&token_params)
        .send()
        .await?;

    if resp.status() >= reqwest::StatusCode::BAD_REQUEST {
        return Err(Box::new(DownloadError {
            description: format!(
                "Failed to get token, status {}: {}",
                resp.status(),
                resp.text().await?
            ),
        }));
    }

    let token = resp.json::<TokenResponse>().await?.access_token;

    let resp = client
        .get("https://www.googleapis.com/drive/v3/files?")
        .query(&[("key", api_key.as_str())])
        .bearer_auth(token.clone())
        .send()
        .await?;

    if resp.status() >= reqwest::StatusCode::BAD_REQUEST {
        return Err(Box::new(DownloadError {
            description: format!(
                "Failed to get file list, status {}: {}",
                resp.status(),
                resp.text().await?
            ),
        }));
    }

    let files = resp.json::<FileListResponse>().await?.files;

    for f in files {
        if f.name == gachi_file_name {
            let id: &String = f.id.borrow();

            let resp = client
                .get(format!("https://www.googleapis.com/drive/v3/files/{}", id))
                .query(&[("key", api_key.as_str()), ("alt", "media")])
                .bearer_auth(token)
                .send()
                .await?;

            if resp.status() >= reqwest::StatusCode::BAD_REQUEST {
                return Err(Box::new(DownloadError {
                    description: format!(
                        "Failed to get file, status {}: {}",
                        resp.status(),
                        resp.text().await?
                    ),
                }));
            }

            return Ok(resp.bytes().await?.to_vec());
        }
    }

    Err(Box::new(DownloadError {
        description: "File was not found on Drive.".to_string(),
    }))
}
