#![forbid(unsafe_code)]

pub use client::ClientFetcher;
pub use generators::{AllTextures, TextureGenerator, Xbrz};

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
    "pack.png",   // Texture pack icon
    VERSION_JSON, // Texture pack version
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

pub enum FileData {
    /// The texture as it's encoded PNG representation.
    EncodedPng(Box<[u8]>),

    /// Texture metadata.
    McMeta(Box<[u8]>),

    /// Actual texture.
    Texture(RgbaImage),
}

impl FileData {
    fn data(&self) -> &[u8] {
        match self {
            Self::EncodedPng(data) => data,
            Self::McMeta(data) => data,
            _ => unreachable!("all textures should have been encoded"),
        }
    }

    fn encode(&mut self) -> Result<(), image::ImageError> {
        if let Self::Texture(texture) = self {
            let mut buf = Vec::with_capacity(texture.len());
            texture.write_with_encoder(PngEncoder::new_with_quality(
                &mut buf,
                CompressionType::Best,
                FilterType::Adaptive,
            ))?;
            let encoded_png = buf.into_boxed_slice();
            *self = Self::EncodedPng(encoded_png);
        }
        Ok(())
    }
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
