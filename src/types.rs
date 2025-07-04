use crate::DIMENSIONS;
use image::RgbaImage;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use rayon::iter::ParallelIterator;
use std::fmt;

pub struct TextureData {
    pub file: FileData,
    pub path: String,
}

impl TextureData {
    pub fn extract(self) -> (FileData, String) {
        (self.file, self.path)
    }
}

impl fmt::Debug for TextureData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Texture")
            .field("path", &self.path.as_str())
            .finish()
    }
}

pub enum FileData {
    /// Texture metadata
    McMeta(Box<[u8]>),
    /// Actual textures
    Texture(RgbaImage),
}

impl FileData {
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsRef<[u8]> for FileData {
    fn as_ref(&self) -> &[u8] {
        // This entire method gets compiled into a couple instructions.
        // My guess is that rust's layout optimization aligns the backing
        // pointers of all variants, so getting the raw bytes
        // is always the same operation.
        match self {
            Self::McMeta(tmp) => tmp,
            Self::Texture(tmp) => tmp,
        }
    }
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
