#![allow(unused)]
#![forbid(unsafe_code)]

//pub use generators::{AllTextures, SingleTexture, TextureGenerator, Xbrz};

mod client;
//mod generators;
mod kdmap;
mod manifest;

const CLIENT_JAR: &str = "client.jar";
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

pub fn testing() -> Result<Box<[u8]>, WoolError> {
    manifest::get_client_jar_bytes(Version::Release)
}

pub struct TextureData {
    pub file: FileData,
    pub path: String,
}

impl TextureData {
    pub fn extract(self) -> (FileData, String) {
        (self.file, self.path)
    }
}

pub enum FileData {
    /// Texture metadata.
    McMeta(Box<[u8]>),

    /// Actual texture.
    Texture(image::RgbaImage),
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
    Utf8(#[from] std::string::FromUtf8Error),

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
    Exact(&'a str),
    Release,
    Snapshot,
}
