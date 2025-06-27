use crate::manifest::{Version, get_client_jar_as_bytes};
use crate::{PNG_EXT, TARGET_DIR, VERSION_JSON};
use std::io::{Cursor, copy};
use vfs::{MemoryFS, VfsPath};
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

    /// Convert contents of `self` into an in-memory, virtual filesystem.
    /// The returned [`VfsPath`] represents the root of said filesystem.
    pub fn into_virt_mem(self) -> VfsPath {
        self.into()
    }

    /// What the fuck do you think this does.
    pub fn print_version_json(&self) {
        let copy = ClientJar::from(self.0.clone());
        let json = copy
            .into_virt_mem()
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .find(|path| path.as_str().ends_with(VERSION_JSON))
            .unwrap()
            .read_to_string()
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        println!("{}", json);
        println!("{:#?}", parsed.as_object().unwrap());
    }
}

impl From<Box<[u8]>> for ClientJar {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
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
                    let is_png = path.ends_with(PNG_EXT);
                    let is_in_target_dir = TARGET_DIR
                        .into_iter()
                        .any(|target_dir| path.contains(target_dir));
                    let is_version_json = path.ends_with(VERSION_JSON);
                    (is_png && is_in_target_dir) || is_version_json
                })
            {
                let path = virt_root.join(path).unwrap();
                // Ensure the file we're about to write has somewhere to be written to.
                path.parent().create_dir_all().unwrap();
                let mut virt_file = path.create_file().unwrap();
                copy(&mut zipped_file, &mut virt_file).unwrap();
            }
        }
        virt_root
    }
}
