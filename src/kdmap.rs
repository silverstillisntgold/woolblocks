use image::RgbaImage;
use kiddo::{
    dist::SquaredEuclidean, kd_tree::KdTree, leaf_strategy::VecOfArenas, stem_strategy::Eytzinger,
};
use rayon::iter::ParallelIterator;

/// How many nuts do we got?
const DIMENSIONS: usize = 3;

/// Whole lotta nuts.
type KdeezNuts =
    KdTree<f64, usize, Eytzinger, VecOfArenas<f64, usize, DIMENSIONS, 32>, DIMENSIONS, 32>;

/// Kinda like a HashMap, but specifically maps RGB values to a representative [`RgbaImage`].
pub struct KdMap {
    keys: KdeezNuts,
    values: Box<[RgbaImage]>,
}

impl<T> From<T> for KdMap
where
    T: ParallelIterator<Item = ([f64; DIMENSIONS], RgbaImage)>,
{
    fn from(value: T) -> Self {
        let (keys, values) = value.collect::<(Box<_>, Box<_>)>();

        let keys = KdeezNuts::new_from_slice_parallel(&keys)
            .expect("initialization of `KdMap` shouldn't fail with our configuration");

        assert_eq!(
            keys.size(),
            values.len(),
            "`keys` and `values` should have the same length"
        );

        Self { keys, values }
    }
}

impl KdMap {
    /// Find the RGB value which is "most similar" to `rgb_query`.
    ///
    /// Similarity is determined via the Euclidean difference between RGB values.
    pub fn find_most_similar(&self, rgb_query: &[f64; DIMENSIONS]) -> &RgbaImage {
        let index = self
            .keys
            .query(rgb_query)
            .nearest_one::<SquaredEuclidean<f64>>()
            .execute()
            .item;

        &self.values[index]
    }
}
