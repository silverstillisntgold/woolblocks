use image::RgbaImage;
use kiddo::{
    dist::SquaredEuclidean,
    kd_tree::{ConstructionError, KdTree},
    leaf_strategy::VecOfArenas,
    stem_strategy::Eytzinger,
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

impl KdMap {
    /// Just read the function name.
    pub fn try_from_par_iter<T>(iter: T) -> Result<Self, ConstructionError>
    where
        T: ParallelIterator<Item = ([f64; DIMENSIONS], RgbaImage)>,
    {
        let (keys_source, values) = iter.collect::<(Box<_>, Box<_>)>();

        let keys = KdeezNuts::new_from_slice_parallel(&keys_source)?;
        assert_eq!(
            keys.size(),
            values.len(),
            "`keys` and `values` should always have the same length"
        );

        Ok(Self { keys, values })
    }

    /// You'll never guess what this does...
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
