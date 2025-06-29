use crate::manifest::{Version, get_client_jar_as_bytes};
use crate::{PNG_EXT, TARGET_DIR, Texture, VERSION_JSON};
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Wraps the raw bytes of a client jar.
pub struct ClientJar(Box<[u8]>);

impl ClientJar {
    /// Create a new [`ClientJar`] from the user-provided `version_id`.
    ///
    /// The program will panic if no match for `version_id` is found.
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

    /// Returns a vector of the textures and the resource pack version.
    pub fn parse(self) -> (Vec<Texture>, u64) {
        self.into()
    }
}

impl From<Box<[u8]>> for ClientJar {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}

impl From<ClientJar> for (Vec<Texture>, u64) {
    fn from(value: ClientJar) -> Self {
        let reader = Cursor::new(&value.0);
        let mut zip = ZipArchive::new(reader).unwrap();
        let mut textures = Vec::with_capacity(zip.len());
        let mut resource_pack_version = 0;
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
                    let is_png = path.ends_with(PNG_EXT);
                    let is_in_target_dir = TARGET_DIR
                        .into_iter()
                        .any(|target_dir| path.contains(target_dir));
                    let is_version_json = path.ends_with(VERSION_JSON);
                    (is_png && is_in_target_dir) || is_version_json
                })
            {
                println!("{}", path);
                match path.ends_with(PNG_EXT) {
                    true => {
                        let mut buf = Vec::with_capacity(zipped_file.size() as usize);
                        zipped_file.read_to_end(&mut buf).unwrap();
                        let img =
                            image::load_from_memory_with_format(&buf, image::ImageFormat::Png)
                                .unwrap()
                                .into_rgba8();
                        textures.push(Texture { img, path });
                    }
                    false => {
                        let mut buf = String::with_capacity(zipped_file.size() as usize);
                        zipped_file.read_to_string(&mut buf).unwrap();
                        // It's fucking beautiful.
                        resource_pack_version = serde_json::from_str::<serde_json::Value>(&buf)
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
                    }
                }
            }
        }
        textures.shrink_to_fit();
        (textures, resource_pack_version)
    }
}
