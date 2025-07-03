use image::RgbaImage;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use rayon::iter::ParallelIterator;
use std::fmt;

const DIMENSIONS: usize = 3;

pub enum Version<'a> {
    Custom(&'a str),
    Release,
    Snapshot,
}

#[allow(unused)]
pub enum FileData {
    /// png.mcmeta files store information about animated
    /// or otherwise configurable textures.
    PngMcMeta(Box<[u8]>),
    /// The actual textures of the resource pack.
    Texture(RgbaImage),
    /// File containing the resource pack version.
    /// There only ever be one of these.
    VersionJson(Box<[u8]>),
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
        // pointers of all three variants, so getting the raw bytes
        // is always the same operation.
        match self {
            Self::PngMcMeta(tmp) => tmp,
            Self::Texture(tmp) => tmp,
            Self::VersionJson(tmp) => tmp,
        }
    }
}

#[allow(unused)]
pub struct TextureV2 {
    pub data: FileData,
    pub path: String,
}

impl TextureV2 {
    pub fn is_png_mcmeta(&self) -> bool {
        match self.data {
            FileData::PngMcMeta(_) => true,
            _ => false,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }
}

pub struct Texture {
    pub img: RgbaImage,
    pub path: String,
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
    /// Find the [`RgbaImage`] whose overall color is "closest" to that of `query`.
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
        let (keys_src, values) = value.collect::<(Vec<_>, Vec<_>)>();
        let keys = ImmutableKdTree::new_from_slice(&keys_src);
        assert_eq!(keys.size(), values.len());
        Self { keys, values }
    }
}
