use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use zip::read::*;

fn download_archive() -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(r"GSP.zip")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    Ok(buffer)
}

pub fn mp3() -> HashMap<String, Vec<u8>> {
    let mut files = HashMap::new();

    let mut zip_data = Vec::new();
    match download_archive() {
        Ok(data) => zip_data = data,
        Err(_) => return HashMap::new(),
    }

    let mut archive = ZipArchive::new(Cursor::new(zip_data)).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let mut buffer = Vec::new();

        match std::io::copy(&mut file, &mut buffer) {
            Ok(_) => (),
            Err(_) => continue,
        }

        files.insert(name.to_str().unwrap().to_string(), buffer);
    }

    return files;
}
