/*!
Documentation goes here.
*/

//#![deny(missing_docs)]

mod client;
mod generator;
mod impls;
mod manifest;

use camino::Utf8PathBuf;
use image::RgbaImage;
use kiddo::{ImmutableKdTree, Manhattan};
use std::fmt;

pub use client::ClientJar;
pub use generator::TextureGenerator;
pub use impls::*;

const DIMENSIONS: usize = 3;
const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const PNG_EXT: &str = ".png";
const SIZE: u32 = 16;
/// Directories which will have their textures replaced.
const TARGET_DIR: &[&str] = &["/block/", "/entity/", "/item/", "/trims/"];
const VERSION_JSON: &str = "version.json";

pub struct KdMap {
    keys: ImmutableKdTree<f64, DIMENSIONS>,
    values: Vec<RgbaImage>,
}

impl<T> From<T> for KdMap
where
    T: Iterator<Item = ([f64; DIMENSIONS], RgbaImage)>,
{
    fn from(value: T) -> Self {
        let (keys_src, values) = value.collect::<(Vec<_>, Vec<_>)>();
        let keys = ImmutableKdTree::new_from_slice(&keys_src);
        assert_eq!(keys.size(), values.len());
        Self { keys, values }
    }
}

impl KdMap {
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get_nearest(&self, query: &[f64; DIMENSIONS]) -> &RgbaImage {
        let index = self.keys.nearest_one::<Manhattan>(query).item as usize;
        &self.values[index]
    }
}

pub struct Texture {
    img: RgbaImage,
    path: Utf8PathBuf,
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
