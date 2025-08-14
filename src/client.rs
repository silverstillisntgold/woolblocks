use crate::manifest::get_client_jar_as_bytes;
use crate::types::{FileData, TextureData};
use crate::{CLIENT_JAR, EXCLUSIONS, INCLUSIONS, MCMETA_EXT, PNG_EXT, VERSION_JSON, Version};
use image::{ImageFormat, load_from_memory_with_format};
use std::fs;
use std::io::{Cursor, Read, Write};
use zip::ZipArchive;

/// Wraps the raw bytes of a client jar.
pub struct ClientJar(Box<[u8]>);

impl ClientJar {
    /// Create a new [`ClientJar`] from the user-provided `version_id`.
    ///
    /// The program will panic if no match for `version_id` is found.
    pub fn new(version_id: Version) -> Self {
        get_client_jar_as_bytes(version_id).into()
    }

    /// Return a [`Vec`] containing all to-be-replaced textures and their
    /// associated paths, as well as the resource pack version.
    pub fn parse(self) -> (Vec<TextureData>, u64) {
        self.into()
    }

    /// Write client jar data to `jar_dir`.
    pub fn write(&self, jar_path: String) {
        let jar_name = jar_path + CLIENT_JAR;
        let mut file = fs::File::create(jar_name).unwrap();
        file.write_all(&self.0).unwrap();
    }
}

impl From<Box<[u8]>> for ClientJar {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}

impl From<ClientJar> for (Vec<TextureData>, u64) {
    fn from(value: ClientJar) -> Self {
        let reader = Cursor::new(value.0);
        let mut zip = ZipArchive::new(reader).unwrap();
        let mut textures = Vec::with_capacity(zip.len());
        let mut resource_pack_version = 0;
        for file_number in 0..zip.len() {
            let mut zipped_file = zip.by_index(file_number).unwrap();
            if let Some(path) = zipped_file
                .enclosed_name()
                .map(|path| path.into_os_string().into_string().unwrap())
                .filter(|path| {
                    let is_included = INCLUSIONS.iter().any(|s| path.contains(s));
                    let is_excluded = EXCLUSIONS.iter().any(|s| path.contains(s));
                    is_included && !is_excluded
                })
            {
                // Why the fuck doesn't .size() return a usize?
                let mut buf = Vec::with_capacity(zipped_file.size() as usize);
                let len = zipped_file.read_to_end(&mut buf).unwrap();
                // This being true guarantees no reallocations were made during reading.
                assert_eq!(buf.len(), len);
                let (_, extension) = path.rsplit_once('.').unwrap_or_default();
                match extension {
                    PNG_EXT => {
                        let img = load_from_memory_with_format(&buf, ImageFormat::Png)
                            .unwrap()
                            .into_rgba8();
                        let file = FileData::Texture(img);
                        textures.push(TextureData { file, path });
                    }
                    MCMETA_EXT => {
                        let file = FileData::McMeta(buf.into_boxed_slice());
                        textures.push(TextureData { file, path });
                    }
                    _ => {
                        if path.ends_with(VERSION_JSON) {
                            assert_eq!(
                                resource_pack_version, 0,
                                "this branch should only be reachable a single time:\
                                before the resource pack version has been read"
                            );
                            resource_pack_version = version_json_to_version(buf);
                        }
                    }
                }
            }
        }
        // As of version 1.21.7, the initial allocation is over 26,000
        // elements but the final length is only around 2,500.
        textures.shrink_to_fit();
        (textures, resource_pack_version)
    }
}

/// Extract the resource pack version from the raw bytes of `version.json`.
fn version_json_to_version(buf: Vec<u8>) -> u64 {
    let json = String::from_utf8(buf).unwrap();
    // It's fucking beautiful.
    serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&json)
        .unwrap()
        .get("pack_version")
        .unwrap()
        .as_object()
        .unwrap()
        .get("resource")
        .unwrap()
        .as_u64()
        .unwrap()
}
