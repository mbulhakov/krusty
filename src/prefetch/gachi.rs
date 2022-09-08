use bytes::{BufMut, Bytes, BytesMut};
use std::boxed::Box;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use zip::read::ZipArchive;
use std::env;

pub async fn ogg() -> Result<HashMap<String, Bytes>, Box<dyn Error>> {
    let mut result = HashMap::new();

    let zip_path = env::var("GACHI_FILE_PATH")?;
    let mut archive = ZipArchive::new(File::open(zip_path)?)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.is_dir() {
            continue;
        }

        let name = file
            .enclosed_name()
            .and_then(|x| x.to_str())
            .map(|x| x.to_string());

        let name = match name {
            None => continue,
            Some(n) => n,
        };

        let mut buf = BytesMut::new().writer();
        match std::io::copy(&mut file, &mut buf) {
            Ok(_) => (),
            Err(err) => {
                log::error!("Failed to copy '{}': {}", name, err);
                continue;
            }
        }

        result.insert(name, buf.into_inner().freeze());
    }

    Ok(result)
}
