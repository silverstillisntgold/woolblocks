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

pub struct KdMap {
    keys: KdeezNuts,
    values: Box<[RgbaImage]>,
}

impl<T> From<T> for KdMap
where
    T: ParallelIterator<Item = ([f64; DIMENSIONS], RgbaImage)>,
{
    fn from(value: T) -> Self {
        // The "average" RGB value of the image is represented by `keys_source`,
        // and the image itself is represented by `values`.
        let (keys_source, values) = value.collect::<(Box<_>, Box<_>)>();

        let keys = KdeezNuts::new_from_slice_parallel(&keys_source)
            .expect("initialization of `KdMap` shouldn't fail with our configuration");
        assert_eq!(
            keys.size(),
            values.len(),
            "`keys` and `values` should always have the same length"
        );

        Self { keys, values }
    }
}

impl KdMap {
    /// Find the RGB value which is "most similar" to `query`.
    ///
    /// Currently, similarity is determined via the Euclidean difference between RGB values.
    pub fn find_most_similar(&self, query: &[f64; DIMENSIONS]) -> &RgbaImage {
        let index = self
            .keys
            .query(query)
            .nearest_one::<SquaredEuclidean<f64>>()
            .execute()
            .item;

        &self.values[index]
    }
}
