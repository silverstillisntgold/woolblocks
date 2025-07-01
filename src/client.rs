use crate::manifest::get_client_jar_as_bytes;
use crate::types::{Texture, Version};
use crate::{EXCLUSIONS, INCLUSIONS, PNG_EXT};
use image::{ImageFormat, load_from_memory_with_format};
use std::io::{Cursor, Read};
use zip::ZipArchive;

const VERSION_JSON: &str = "version.json";

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
        let reader = Cursor::new(value.0);
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
                    // Source directories contain tons of useless shit.
                    let is_png = path.ends_with(PNG_EXT);
                    let is_gay = EXCLUSIONS
                        .into_iter()
                        .any(|exclusion| path.contains(exclusion));
                    // We are only interested in a specific subset of directories.
                    let is_in_target_dir = INCLUSIONS
                        .into_iter()
                        .any(|target_dir| path.contains(target_dir));
                    // So our generated resource pack has a nice icon :).
                    let is_pack = path.ends_with("pack.png");
                    // Needed to avoid client being pissy about incorrect
                    // resource pack version (whiny bitch frfr).
                    let is_version_json = path.ends_with(VERSION_JSON);
                    !is_gay && ((is_png && is_in_target_dir) || is_pack || is_version_json)
                })
            {
                // Why the fuck doesn't .size() return a usize?
                let mut buf = Vec::with_capacity(zipped_file.size() as usize);
                let len = zipped_file.read_to_end(&mut buf).unwrap();
                // This being true guarantees no reallocations during reading.
                assert_eq!(buf.len(), len);
                match path.ends_with(PNG_EXT) {
                    true => {
                        let img = load_from_memory_with_format(&buf, ImageFormat::Png)
                            .unwrap()
                            .into_rgba8();
                        textures.push(Texture { img, path });
                    }
                    false => {
                        assert_eq!(
                            resource_pack_version, 0,
                            "this branch should only be reachable a single time:\
                            before the resource pack version has been read"
                        );
                        let buf = String::from_utf8(buf).unwrap();
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
        // As of version 1.21.6, the initial allocation is over 26,000
        // elements but the final length is just under 2,500.
        textures.shrink_to_fit();
        (textures, resource_pack_version)
    }
}
