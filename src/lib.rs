#![forbid(unsafe_code)]

pub use generators::{AllTextures, SingleTexture, TextureGenerator, Xbrz};

mod client;
mod generators;
mod manifest;
mod types;

#[derive(Debug, thiserror::Error)]
pub enum WoolError {
    #[error(transparent)]
    Http(#[from] ureq::Error),

    #[error("")]
    MismatchSha1Manifest,

    #[error("")]
    MismatchSha1Data,

    #[error("")]
    MismatchVersion,

    #[error("invalid minnecraft version")]
    InvalidVersion,

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub enum Version<'a> {
    Custom(&'a str),
    Release,
    Snapshot,
}

const CLIENT_JAR: &str = "client.jar";
/// Used in internal KdMap implementation.
const DIMENSIONS: usize = 3;
const EXCLUSIONS: &[&str] = &[
    "/color_palettes/", // Subdirectory of "trims"
];
const INCLUSIONS: &[&str] = &[
    "/block/",      // Block textures
    "/entity/",     // Entity textures
    "/item/",       // Handheld item textures
    "/mob_effect/", // Status effect textures
    "pack.png",     // Texture pack icon
    "/trims/",      // Armor trim textures
    "version.json", // Texture pack version
];
const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const MCMETA_EXT: &str = "mcmeta";
const PACK_MCMETA: &str = "pack.mcmeta";
const PNG_EXT: &str = "png";
const SIZE: u32 = 16;
const VERSION_JSON: &str = INCLUSIONS.last().unwrap();
