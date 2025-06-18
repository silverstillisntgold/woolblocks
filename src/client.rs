use crate::manifest::{Version, get_client_jar_as_bytes};
use crate::{HOME_DIR_LEN, SOURCE_DIR, TARGET_DIR};
use camino::Utf8PathBuf;
use image::codecs::png::*;
use std::ffi::OsString;
use std::io::{Cursor, copy};
use std::path::PathBuf;
use vfs::{MemoryFS, VfsPath};
use zip::ZipArchive;

/// Stores the raw bytes of a client jar.
pub struct ClientJar(Box<[u8]>);

impl From<Box<[u8]>> for ClientJar {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}

impl ClientJar {
    pub fn new(version_id: &str) -> Self {
        get_client_jar_as_bytes(Version::Custom(version_id)).into()
    }

    pub fn new_release() -> Self {
        get_client_jar_as_bytes(Version::Release).into()
    }

    pub fn new_snapshot() -> Self {
        get_client_jar_as_bytes(Version::Snapshot).into()
    }

    pub fn print(self) {
        let virt_root = self.extract_to_virt_fs();
        let mut v = virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .map(|p| p.as_str().to_owned())
            .collect::<Vec<_>>();
        v.sort_unstable();
        v.sort_unstable_by_key(|s| s.len());
        println!("{:#?}", v);
        let wow = virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .find(|x| x.as_str().contains("version.json"))
            .unwrap();
        let xd = wow.read_to_string().unwrap();
        println!("{}", xd);
    }

    /// Extracts all files from the contents of `self` into a virtual,
    /// in-memory filesystem. Returns the root of said filesystem.
    fn extract_to_virt_fs(self) -> VfsPath {
        let reader = Cursor::new(self.0);
        let mut zip = ZipArchive::new(reader).unwrap();
        let virt_root = VfsPath::new(MemoryFS::new());
        for file_number in 0..zip.len() {
            let mut zipped_file = zip.by_index(file_number).unwrap();
            assert!(
                zipped_file.is_file(),
                "`ZipArchive::by_index` should only provide files"
            );
            if let Some(path) = zipped_file
                .enclosed_name()
                .map(PathBuf::into_os_string)
                .map(OsString::into_string)
                .map(Result::unwrap)
            {
                let path = virt_root.join(path).unwrap();
                path.parent().create_dir_all().unwrap();
                let mut virt_file = path.create_file().unwrap();
                copy(&mut zipped_file, &mut virt_file).unwrap();
            }
        }
        virt_root
    }
}

pub trait TextureGenerator {
    fn make_new(&self) -> u64 {
        69_420
    }
}

pub struct Texture2 {
    path: Utf8PathBuf,
    texture_bytes: Box<[u8]>,
}

pub fn client_jar_into_sources(client_data: Box<[u8]>) -> Box<[Texture2]> {
    let reader = Cursor::new(client_data);
    ZipArchive::new(reader)
        .unwrap()
        .extract(&SOURCE_DIR[..HOME_DIR_LEN])
        .unwrap();

    walkdir::WalkDir::new(&SOURCE_DIR[..HOME_DIR_LEN])
        .into_iter()
        .map(|dir| {
            let dir = dir.unwrap();
            let ft = dir.file_type();
            let path = Utf8PathBuf::from_path_buf(dir.into_path()).unwrap();
            (path, ft)
        })
        .filter_map(|(dir, ft)| {
            match dir.starts_with(SOURCE_DIR) && ft.is_file() && dir.as_str().ends_with(".png") {
                true => Some(dir),
                false => None,
            }
        })
        .filter(|path| {
            TARGET_DIR
                .into_iter()
                .any(|target| path.components().any(|c| c.as_str().eq(*target)))
        })
        .map(|dir| {
            let data = std::fs::read(dir.as_std_path()).unwrap().into_boxed_slice();
            Texture2 {
                path: dir,
                texture_bytes: data,
            }
        })
        .collect()

    /*paths.sort_unstable_by_key(|path| path.components().count());
    println!("{:#?}", paths);

    let wool = paths
        .iter()
        .find(|path| path.file_name().unwrap().contains("wool"))
        .unwrap();
    println!("{}", wool);
    let mut img = image::ImageReader::open(wool).unwrap();
    img.set_format(image::ImageFormat::Png);
    let wool_img = img.decode().unwrap().to_rgba8();

    let f = std::fs::File::create("rizz.ler.real").unwrap();
    let enc = PngEncoder::new_with_quality(f, CompressionType::Best, FilterType::default());
    wool_img.write_with_encoder(enc).unwrap();*/
}
