use crate::prefetch::download::download_gachi_from_drive;
use bytes::{BufMut, Bytes, BytesMut};
use std::boxed::Box;
use std::collections::HashMap;
use std::error::Error;
use std::io::Cursor;
use zip::read::ZipArchive;

pub async fn ogg() -> Result<HashMap<String, Bytes>, Box<dyn Error>> {
    let mut result = HashMap::new();

    let mut archive = ZipArchive::new(Cursor::new(download_gachi_from_drive().await?))?;
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

    return Ok(result);
}
