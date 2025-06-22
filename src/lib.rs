use camino::Utf8PathBuf;
use image::RgbaImage;
use std::fmt::{Debug, Formatter, Result};

type HashMap<K, V> = std::collections::HashMap<K, V, foldhash::quality::RandomState>;

pub mod client;
mod generator;
mod impls;
mod manifest;

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
/// Directories which will have the majority of their textures replaced.
const TARGET_DIR: &[&str] = &["/block/", "/entity/", "/item/", "/trims/"];
const SIZE: u32 = 16;

pub struct Texture {
    img: RgbaImage,
    path: Utf8PathBuf,
}

impl Debug for Texture {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Texture").field("path", &self.path).finish()
    }
}
