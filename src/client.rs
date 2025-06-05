use crate::manifest::get_client_jar_as_bytes as gcjab; // lol
use std::io::Cursor;
use zip::ZipArchive;

pub fn output_directories(client_data: Box<[u8]>) {
    let cursor = Cursor::new(client_data);
    ZipArchive::new(cursor)
        .expect("failed to unzip")
        .extract(".wool/client")
        .expect("failed to extract");
}

/// Given the raw data of a client jar, ...
pub fn client_jar_into_sources(client_data: Box<[u8]>) {
    let cursor = Cursor::new(client_data);
    ZipArchive::new(cursor)
        .expect("failed to unzip")
        .extract(".wool/client")
        .expect("failed to extract");
}
