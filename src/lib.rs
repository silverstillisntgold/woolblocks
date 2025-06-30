mod client;
mod generators;
mod manifest;
mod types;

pub use generators::{AllTextures, SingleTexture, TextureGenerator, Xbrz};
pub use types::Version;

const PNG_EXT: &str = ".png";
/// Directories which will have their textures replaced.
const TARGET_DIR: &[&str] = &["/block/", "/entity/", "/item/", "/trims/"];
