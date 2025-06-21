use crate::manifest::{Version, get_client_jar_as_bytes};
use crate::{TARGET_DIR, Texture};
use camino::Utf8PathBuf;
use image::codecs::png::*;
use image::*;
use rayon::prelude::*;
use std::collections::HashMap as HashMapCore;
use std::io::{Cursor, copy};
use vfs::{MemoryFS, VfsFileType, VfsPath};
use zip::ZipArchive;

type HashMap<K, V> = HashMapCore<K, V, foldhash::quality::RandomState>;

/// Wraps the raw bytes of a client jar.
pub struct ClientJar(Box<[u8]>);

impl From<Box<[u8]>> for ClientJar {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}

impl ClientJar {
    /// Create a new [`ClientJar`] from the user-provided `version_id`.
    ///
    /// The program will panic if `version_id` match isn't found.
    pub fn new(version_id: &str) -> Self {
        get_client_jar_as_bytes(Version::Custom(version_id)).into()
    }

    /// Create a new [`ClientJar`] from the latest release version available.
    pub fn new_release() -> Self {
        get_client_jar_as_bytes(Version::Release).into()
    }

    /// Create a new [`ClientJar`] from the latest snapshot version available.
    pub fn new_snapshot() -> Self {
        get_client_jar_as_bytes(Version::Snapshot).into()
    }

    /// Convert contents of `self` into an in-memory, virtual filesystem.
    /// The returned [`VfsPath`] represents the root of said filesystem.
    pub fn into_virt_mem(self) -> VfsPath {
        self.into()
    }

    pub fn print_json_data(&self) {
        let copy = ClientJar::from(self.0.clone());
        let json = copy
            .into_virt_mem()
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .find(|path| path.as_str().ends_with("version.json"))
            .unwrap()
            .read_to_string()
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        println!("{}", json);
        println!("{:#?}", parsed.as_object().unwrap());
    }
}

impl From<ClientJar> for VfsPath {
    fn from(value: ClientJar) -> Self {
        // Why worry about your operating system caching your filesystem
        // when you can just force everything into RAM :).
        let virt_root = VfsPath::new(MemoryFS::new());
        let reader = Cursor::new(value.0);
        let mut zip = ZipArchive::new(reader).unwrap();
        for file_number in 0..zip.len() {
            let mut zipped_file = zip.by_index(file_number).unwrap();
            assert!(
                zipped_file.is_file(),
                "`ZipArchive::by_index` should only provide files"
            );
            if let Some(path) = zipped_file
                .enclosed_name()
                // Strings are just simpler to work with here.
                .map(|path| path.into_os_string().into_string().unwrap())
                .filter(|path| {
                    let is_png = path.ends_with(".png");
                    let is_in_target_dir = TARGET_DIR
                        .into_iter()
                        .any(|target_dir| path.contains(target_dir));
                    let is_version_json = path.ends_with("version.json");
                    (is_png && is_in_target_dir) || is_version_json
                })
                // Eliminate some goofy ah textures.
                .filter(|path| !path.contains("test") && !path.contains("debug"))
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
    fn compute_texture_avg_mapping(&self, textures: Vec<Texture>) -> HashMap<Rgba<u8>, RgbaImage>;

    /// Convert the raw data of all images within `virt_root` into textures,
    /// and extract the resource pack version from its `version.json`.
    fn extract_data(&self, virt_root: VfsPath) -> (Vec<Texture>, u64) {
        let textures = virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .filter_map(|path| {
                let md = path.metadata().unwrap();
                let is_file = md.file_type == VfsFileType::File;
                let is_png = path.as_str().ends_with(".png");
                (is_file && is_png).then(|| {
                    // As ugly as it is this seems to be the best way
                    // to convert `VfsPath` files into buffers.
                    let buf = {
                        let capacity = md.len as usize;
                        let mut writer = vec![0; capacity];
                        let mut reader = path.open_file().unwrap();
                        copy(&mut reader, &mut writer).unwrap();
                        writer
                    };
                    let dyn_img = load_from_memory_with_format(&buf, ImageFormat::Png).unwrap();
                    let img = dyn_img.to_rgba8();
                    let path = Utf8PathBuf::from(path.as_str());
                    Texture { img, path }
                })
            })
            .collect();

        // It's fucking beautiful.
        let json_string = virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .find(|path| path.as_str().ends_with("version.json"))
            .unwrap()
            .read_to_string()
            .unwrap();
        let resource_pack_version = serde_json::from_str::<serde_json::Value>(&json_string)
            .unwrap()
            .as_object()
            .unwrap()
            .get("pack_version")
            .unwrap()
            .as_object()
            .unwrap()
            .get("resource")
            .unwrap()
            .as_number()
            .unwrap()
            .as_u64()
            .unwrap();

        (textures, resource_pack_version)
    }

    fn write(&self, textures: Vec<Texture>, target_dir: &str) {
        todo!()
    }
}

pub struct Wool;
impl TextureGenerator for Wool {
    fn compute_texture_avg_mapping(&self, textures: Vec<Texture>) -> HashMap<Rgba<u8>, RgbaImage> {
        todo!()
    }
}
