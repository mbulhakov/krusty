use std::boxed::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use zip::read::*;

fn download_archive() -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(r"C:\Users\Maksym\Documents\GSP.zip")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    Ok(buffer)
}

pub fn mp3() -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
    let mut result = HashMap::new();

    let mut archive = ZipArchive::new(Cursor::new(download_archive()?))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.is_dir() {
            continue;
        }

        let name = file
            .enclosed_name()
            .and_then(|x| x.to_str())
            .and_then(|x| Some(x.to_string()));

        let name = match name {
            None => continue,
            Some(n) => n,
        };

        let mut buffer = Vec::new();
        match std::io::copy(&mut file, &mut buffer) {
            Ok(_) => (),
            Err(err) => {
                log::error!("Failed to copy '{}': {}", name, err);
                continue;
            }
        }

        result.insert(name, buffer);
    }

    return Ok(result);
}
