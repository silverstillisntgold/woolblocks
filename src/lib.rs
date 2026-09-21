#![forbid(unsafe_code)]

pub use client::ClientFetcher;
pub use generators::{AllTextures, TextureGenerator};

use camino::Utf8PathBuf;
use image::{
    RgbaImage,
    codecs::png::{CompressionType, FilterType, PngEncoder},
};

mod client;
mod generators;
mod kdmap;
mod manifest;

const EXCLUSIONS: &[&str] = &[
    "color_palettes", // Obnoxious subdirectory of "trims"
];
const INCLUSIONS: &[&str] = &[
    // Paths: Both as source and as target textures
    "block",      // Block textures
    "entity",     // Entity textures
    "item",       // Handheld item textures
    "mob_effect", // Status effect textures
    "trims",      // Armor trim textures
    // Files: These fuckers live at root so we have to specify them
    "pack.png",     // Texture pack icon
    "version.json", // Texture pack version
];

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const OUTPUT_DIR: &str = "generated";
const PACK_MCMETA: &str = "pack.mcmeta";
const SIZE: u32 = 16;
const VERSION_JSON: &str = "version.json";

const JSON_EXT: &str = "json";
const MCMETA_EXT: &str = "mcmeta";
const PNG_EXT: &str = "png";
const ZIP_EXT: &str = "zip";

pub struct TextureData {
    file: FileData,

    path: Utf8PathBuf,
}

impl TextureData {
    fn file_data(&self) -> Result<Vec<u8>, WoolError> {
        match &self.file {
            FileData::Texture(texture) => {
                let mut buf = Vec::with_capacity(texture.len());

                let enc = PngEncoder::new_with_quality(
                    &mut buf,
                    CompressionType::Best,
                    FilterType::Adaptive,
                );
                texture.write_with_encoder(enc)?;

                Ok(buf)
            }

            FileData::McMeta(metadata) => Ok(metadata.clone().into_vec()),
        }
    }
}

pub enum FileData {
    /// Texture metadata.
    McMeta(Box<[u8]>),

    /// Actual texture.
    Texture(RgbaImage),
}

#[derive(Debug, thiserror::Error)]
pub enum WoolError {
    #[error(transparent)]
    Http(#[from] ureq::Error),

    #[error("invalid Minecraft version provided by user")]
    InvalidVersion,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    KdTree(#[from] kiddo::kd_tree::ConstructionError),

    #[error(transparent)]
    Mismatch(#[from] MismatchError),

    #[error(transparent)]
    Utf8(#[from] camino::FromPathBufError),

    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
}

#[derive(Debug, thiserror::Error)]
pub enum MismatchError {
    #[error("client JAR SHA-1 does not match manifest")]
    ClientJarSha1,

    #[error("`keys` and `values` should always have the same length")]
    KdMapLength,

    #[error("version manifest SHA-1 does not match version index")]
    VersionManifestSha1,

    #[error("version ID does not match requested version")]
    VersionId,
}

pub enum Version<'a> {
    /// A specific Minecaft version.
    Exact(&'a str),

    /// The latest Minecraft release.
    Release,

    /// The latest Minecraft snapshot.
    Snapshot,
}
