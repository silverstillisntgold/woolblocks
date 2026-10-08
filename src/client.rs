use crate::{
    EXCLUSIONS, FileData, INCLUSIONS, JSON_EXT, MCMETA_EXT, PNG_EXT, TextureData, VERSION_JSON,
    Version, WoolError, manifest::get_client_jar_bytes,
};
use camino::Utf8PathBuf;
use image::{ImageFormat, load_from_memory_with_format};
use serde::Deserialize;
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Utility for fetching the raw bytes of a client jar.
#[derive(bon::Builder)]
pub struct ClientFetcher<'a> {
    /// File path patterns that should be excluded when building the texture pack list.
    exclusions: Option<&'a [&'a str]>,

    /// File path patterns that should be included when building the texture pack list.
    inclusions: &'a [&'a str],

    /// Version of Minecraft whose textures should be fetched.
    version: Version<'a>,
}

impl<'a> ClientFetcher<'a> {
    /// The latest stable Minecraft version.
    pub fn release() -> Self {
        Self::with_version(Version::Release)
    }

    /// The latest snapshot Minecraft version.
    pub fn snapshot() -> Self {
        Self::with_version(Version::Snapshot)
    }

    /// Provides reasonable default exclusions and inclusions paired with the provided `version`.
    ///
    /// Use [`Self::release`] or [`Self::snapshot`] for the latest Minecraft versions.
    pub fn with_version(version: Version<'a>) -> Self {
        // Builder syntax make me nut low-k.
        Self::builder()
            .exclusions(EXCLUSIONS)
            .inclusions(INCLUSIONS)
            .version(version)
            .build()
    }

    /// Fetches and parses the client jar, returning textures and the major resource pack version.
    #[inline(never)]
    pub fn fetch(self) -> Result<(Box<[TextureData]>, u64), WoolError> {
        let client_jar_bytes = get_client_jar_bytes(self.version)?;

        // Our `zip_reader` needs `Read` + `Seek` traits so we have to
        // wrap the jar byte buffer in a `Cursor`. Kinda gay but whatever.
        let reader = Cursor::new(client_jar_bytes);
        let mut zip_reader = ZipArchive::new(reader)?;

        // Provide a generous upper bound to avoid reallocating during the loop.
        let mut textures = Vec::with_capacity(zip_reader.len());
        let mut pack_version = None;

        // YO DUMBASS WHY DO IT THIS WAY?!
        //
        // Unfortunately, parallelizing reading from a zip file is not really feasible (FUUUUCCCCKKKKK).
        // We always have to hit each entry in the zip, and once we do, we may as well fully process it.
        //
        // I did try to do an initial serial pass which stored paths and cached the entry buffer
        // for use in a second parallel (rayon) path, but that increased runtime by ~3x (bruh).
        // The slowest part is almost certainly the buffer allocation to store each entry,
        // and since this serial approach filters before allocating buffers there's far less memory
        // allocations. The chuds were right: computation is cheap, memory is expensive.
        //
        // Also notice that we don't do any PNG encoding here. We only capture textures,
        // mcmeta, and the `version.json` major pack version.
        for file_number in 0..zip_reader.len() {
            let mut file = zip_reader.by_index(file_number)?;

            if let Some(path) = file.enclosed_name() {
                let path = Utf8PathBuf::try_from(path)?;

                let included = self.inclusions.iter().any(|inclusion| {
                    path.components()
                        .any(|component| component.as_str().eq(*inclusion))
                });
                let excluded = if let Some(exclusions) = self.exclusions {
                    exclusions.iter().any(|exclusion| {
                        path.components()
                            .any(|component| component.as_str().eq(*exclusion))
                    })
                } else {
                    false
                };

                // We do a little skipping :tf:
                if !included || excluded {
                    continue;
                }

                let mut buf = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut buf)?;

                match path.extension() {
                    // Actual textures.
                    Some(PNG_EXT) => {
                        let texture =
                            load_from_memory_with_format(&buf, ImageFormat::Png)?.into_rgba8();
                        let file = FileData::Texture(texture);

                        textures.push(TextureData { file, path });
                    }

                    // McMeta files are required for certain textures (fire, water, etc.)
                    // to not be completely fucked up.
                    Some(MCMETA_EXT) => {
                        let file = FileData::McMeta(buf.into_boxed_slice());

                        textures.push(TextureData { file, path });
                    }

                    // Use `version.json` to acquire the major resource version.
                    Some(JSON_EXT) if path.as_str().ends_with(VERSION_JSON) => {
                        assert!(
                            pack_version.is_none(),
                            "the resource pack version should not have been read yet"
                        );

                        #[derive(Deserialize)]
                        struct VersionJson {
                            pack_version: PackVersion,
                        }

                        #[derive(Deserialize)]
                        struct PackVersion {
                            resource_major: u64,
                        }

                        let parsed_major_version = serde_json::from_slice::<VersionJson>(&buf)?
                            .pack_version
                            .resource_major;

                        pack_version = Some(parsed_major_version);
                    }

                    _ => {
                        // Ignore any other shit.
                    }
                }
            }
        }

        Ok((
            textures.into_boxed_slice(),
            pack_version.expect("all Minecraft client jar's should contain `version.json`"),
        ))
    }
}
