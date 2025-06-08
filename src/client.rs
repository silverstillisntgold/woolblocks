use crate::manifest::get_client_jar_as_bytes as gcjab; // lol
use crate::{HOME_DIR_LEN, SOURCE_DIR, TARGET_DIR};
use camino::{Utf8Path, Utf8PathBuf};
use std::io::Cursor;
use zip::ZipArchive;

pub fn output_directories(client_data: Box<[u8]>) {
    let cursor = Cursor::new(client_data);
    ZipArchive::new(cursor)
        .unwrap()
        .extract(&SOURCE_DIR[..HOME_DIR_LEN])
        .unwrap();
}

/// Given the raw data of a client jar, ...
pub fn client_jar_into_sources(client_data: Box<[u8]>) -> Box<[Utf8PathBuf]> {
    let reader = Cursor::new(client_data);
    ZipArchive::new(reader)
        .unwrap()
        .extract(&SOURCE_DIR[..HOME_DIR_LEN])
        .unwrap();

    let mut paths = walkdir::WalkDir::new(&SOURCE_DIR[..HOME_DIR_LEN])
        .into_iter()
        .map(|dir| {
            let dir = dir.unwrap();
            let ft = dir.file_type();
            let path = Utf8Path::from_path(dir.path()).unwrap().to_owned();
            (path, ft)
        })
        .filter_map(|(dir, ft)| {
            match dir.starts_with(SOURCE_DIR)
                && (ft.is_dir() || (ft.is_file() && dir.as_str().ends_with(".png")))
            {
                true => Some(dir.to_path_buf()),
                false => None,
            }
        })
        .filter(|path| {
            TARGET_DIR
                .into_iter()
                .any(|target| path.components().any(|c| c.as_str().eq(*target)))
        })
        .collect::<Box<_>>();

    paths.sort_unstable_by_key(|path| path.components().count());
    println!("{:#?}", paths);

    let wool = paths
        .iter()
        .find(|path| path.file_name().unwrap().contains("wool"))
        .unwrap();
    println!("{}", wool);
    let mut img = image::ImageReader::open(wool).unwrap();
    img.set_format(image::ImageFormat::Png);
    let wool_img = img.decode().unwrap().to_luma_alpha8();

    paths
}
