use crate::DIMENSIONS;
use image::RgbaImage;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use rayon::iter::ParallelIterator;

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
    /// Texture metadata
    McMeta(Box<[u8]>),
    /// Actual textures
    Texture(RgbaImage),
}

/// Wraps an `ImmutableKdTree` and `Vec`, where each RGB
/// entry in the KdTree is represented by the `RgbaImage` which
/// used that RGB value as a tint.
pub struct KdMap {
    keys: ImmutableKdTree<f64, DIMENSIONS>,
    values: Box<[RgbaImage]>,
}

impl KdMap {
    /// Find the [`RgbaImage`] whose overall color is "closest" to that of `query`.
    #[inline]
    pub fn nearest(&self, query: &[f64; DIMENSIONS]) -> &RgbaImage {
        let index = self.keys.nearest_one::<SquaredEuclidean>(query).item as usize;
        &self.values[index]
    }
}

impl<T> From<T> for KdMap
where
    T: ParallelIterator<Item = ([f64; DIMENSIONS], RgbaImage)>,
{
    #[inline]
    fn from(value: T) -> Self {
        let (keys_src, values) = value.collect::<(Box<_>, Box<_>)>();
        let keys = ImmutableKdTree::new_from_slice(&keys_src);
        assert_eq!(keys.size(), values.len());
        Self { keys, values }
    }
}
