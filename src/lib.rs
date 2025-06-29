mod client;
mod generator;
mod impls;
mod manifest;

use image::RgbaImage;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use rayon::iter::ParallelIterator;
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

pub struct Texture {
    img: RgbaImage,
    path: String,
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Texture")
            .field("path", &self.path.as_str())
            .finish()
    }
}

/// Wraps an `ImmutableKdTree` and `Vec`, where each RGB
/// entry in the KdTree is represented by the `RgbaImage` which
/// used that RGB value as a tint.
pub struct KdMap {
    keys: ImmutableKdTree<f64, DIMENSIONS>,
    values: Vec<RgbaImage>,
}

impl KdMap {
    /// Returns the numbers of elements in the backing [`ImmutableKdTree`] and [`Vec`].
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Finds the [`RgbaImage`] whose overall color is "closest" to that of `query`.
    pub fn nearest(&self, query: &[f64; DIMENSIONS]) -> &RgbaImage {
        let index = self.keys.nearest_one::<SquaredEuclidean>(query).item as usize;
        &self.values[index]
    }

    /// Provides the backing [`RgbaImage`] slice.
    pub fn textures(&self) -> &[RgbaImage] {
        &self.values
    }
}

impl<T> From<T> for KdMap
where
    T: ParallelIterator<Item = ([f64; DIMENSIONS], RgbaImage)>,
{
    fn from(value: T) -> Self {
        let (keys_src, values) = value.collect::<(Vec<_>, Vec<_>)>();
        let keys = ImmutableKdTree::new_from_slice(&keys_src);
        assert_eq!(keys.size(), values.len());
        Self { keys, values }
    }
}
