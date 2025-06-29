mod client;
mod generators;
mod manifest;

use image::RgbaImage;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use rayon::iter::ParallelIterator;
use std::fmt;

pub use generators::{AllTextures, SingleTexture, TextureGenerator, Xbrz};

const DIMENSIONS: usize = 3;
const PNG_EXT: &str = ".png";
/// Directories which will have their textures replaced.
const TARGET_DIR: &[&str] = &["/block/", "/entity/", "/item/", "/trims/"];
const VERSION_JSON: &str = "version.json";

pub enum Version<'a> {
    Custom(&'a str),
    Release,
    Snapshot,
}

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
    /// Return the numbers of elements in the backing [`ImmutableKdTree`] and [`Vec`].
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Find the [`RgbaImage`] whose overall color is "closest" to that of `query`.
    pub fn nearest(&self, query: &[f64; DIMENSIONS]) -> &RgbaImage {
        let index = self.keys.nearest_one::<SquaredEuclidean>(query).item as usize;
        &self.values[index]
    }

    /// Provide the backing [`RgbaImage`] slice.
    pub fn textures(&self) -> &[RgbaImage] {
        &self.values
    }
}

impl<T> From<T> for KdMap
where
    T: ParallelIterator<Item = ([f64; DIMENSIONS], RgbaImage)>,
{
    #[inline]
    fn from(value: T) -> Self {
        let (keys_src, values) = value.collect::<(Vec<_>, Vec<_>)>();
        let keys = ImmutableKdTree::new_from_slice(&keys_src);
        assert_eq!(keys.size(), values.len());
        Self { keys, values }
    }
}
