use crate::TARGET_DIR;
use crate::manifest::{Version, get_client_jar_as_bytes};
use camino::Utf8PathBuf;
use image::codecs::png::*;
use image::*;
use std::io::{Cursor, copy};
use vfs::{MemoryFS, VfsFileType, VfsPath};
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
        v.sort_by_key(|s| s.len());
        println!("entry count: {}", v.len());
        println!("{:#?}", v);
        let json = virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .find(|x| x.as_str().contains("version.json"))
            .unwrap()
            .read_to_string()
            .unwrap();
        println!("version.json contents:");
        println!("{}", json);
    }

    /// Extracts all files from the contents of `self` into a virtual,
    /// in-memory filesystem. Returns the root of said filesystem.
    pub fn extract_to_virt_fs(self) -> VfsPath {
        let reader = Cursor::new(self.0);
        let mut zip = ZipArchive::new(reader).unwrap();
        let virt_root = VfsPath::new(MemoryFS::new());
        for file_number in 0..zip.len() {
            let mut zipped_file = zip.by_index(file_number).unwrap();
            assert!(
                zipped_file.is_file(),
                "'ZipArchive::by_index' should only provide files"
            );
            if let Some(path) = zipped_file
                .enclosed_name()
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

#[derive(Clone)]
pub struct TextureV2 {
    img_gray: GrayAlphaImage,
    img_rgba: RgbaImage,
    path: Utf8PathBuf,
}

impl std::fmt::Debug for TextureV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureV2")
            .field("path", &self.path)
            .finish()
    }
}

pub trait TextureGenerator {
    /// TODO: docs
    fn get_src_textures(&self, dst_textures: &[TextureV2]) -> Vec<TextureV2>;

    /// Returns all textures which will be overridden using the textures
    /// previously computed by [`TextureGenerator::get_src_textures`].
    fn get_dst_textures(&self, virt_root: &VfsPath) -> Vec<TextureV2> {
        virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .filter_map(|path| {
                let md = path.metadata().unwrap();
                (md.file_type == VfsFileType::File && path.as_str().ends_with(".png")).then(|| {
                    let buf = {
                        let capacity = md.len as usize + 1;
                        let mut writer = Vec::with_capacity(capacity);
                        let mut reader = path.open_file().unwrap();
                        copy(&mut reader, &mut writer).unwrap();
                        writer
                    };
                    let path = Utf8PathBuf::from(path.as_str());
                    (buf, path)
                })
            })
            .map(|(buf, path)| {
                let img = load_from_memory_with_format(&buf, ImageFormat::Png).unwrap();
                let img_gray = img.to_luma_alpha8();
                let img_rgba = img.to_rgba8();
                TextureV2 {
                    img_gray,
                    img_rgba,
                    path,
                }
            })
            .collect()
    }
}

pub struct Wool;
impl TextureGenerator for Wool {
    fn get_src_textures(&self, dst_textures: &[TextureV2]) -> Vec<TextureV2> {
        todo!()
    }
}

pub struct All;
impl TextureGenerator for All {
    fn get_src_textures(&self, dst_textures: &[TextureV2]) -> Vec<TextureV2> {
        todo!()
    }
}
