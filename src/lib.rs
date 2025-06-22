mod client;
mod generator;
mod impls;
mod manifest;

use camino::Utf8PathBuf;
use image::RgbaImage;
use std::fmt::{Debug, Formatter, Result};

pub use client::ClientJar;
pub use generator::TextureGenerator;
pub use impls::*;

pub type HashMap<K, V> = std::collections::HashMap<K, V, foldhash::quality::RandomState>;

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const PNG_EXT: &str = ".png";
const SIZE: u32 = 16;
/// Directories which will have their textures replaced.
const TARGET_DIR: &[&str] = &["/block/", "/entity/", "/item/", "/trims/"];

pub struct Texture {
    img: RgbaImage,
    path: Utf8PathBuf,
}

impl Debug for Texture {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Texture").field("path", &self.path).finish()
    }
}

#[inline]
fn rgb_iter(step: usize) -> impl Iterator<Item = (u8, u8, u8)> {
    (0..=u8::MAX)
        .step_by(step)
        .flat_map(move |r| (0..=u8::MAX).step_by(step).map(move |g| (r, g)))
        .flat_map(move |(r, g)| (0..=u8::MAX).step_by(step).map(move |b| (r, g, b)))
}
