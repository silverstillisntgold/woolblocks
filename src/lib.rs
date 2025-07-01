mod client;
mod generators;
mod manifest;
mod types;

pub use generators::{AllTextures, SingleTexture, TextureGenerator, Xbrz};
pub use types::Version;

const EXCLUSIONS: &[&str] = &["/misc/", "/color_palettes/"];
const INCLUSIONS: &[&str] = &["/block/", "/entity/", "/item/", "/trims/"];
const PNG_EXT: &str = ".png";
