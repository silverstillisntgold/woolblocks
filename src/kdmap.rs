use image::RgbaImage;
use kiddo::{
    dist::SquaredEuclidean,
    kd_tree::{ConstructionError, KdTree},
    leaf_strategy::VecOfArenas,
    stem_strategy::Eytzinger,
};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator};

const DIMENSIONS: usize = 3;

type KdeezNuts =
    KdTree<f64, usize, Eytzinger, VecOfArenas<f64, usize, DIMENSIONS, 32>, DIMENSIONS, 32>;

pub struct KdMap {
    keys: KdeezNuts,
    values: Box<[RgbaImage]>,
}

impl KdMap {
    pub fn try_from_parallel_iter<T>(value: T) -> Result<Self, ConstructionError>
    where
        T: IntoParallelIterator,
        T::Iter: IndexedParallelIterator<Item = ([f64; DIMENSIONS], RgbaImage)>,
    {
        const DEFAULT_CAPACITY: usize = 1 << 16;

        let mut keys_source = Vec::with_capacity(DEFAULT_CAPACITY);
        let mut values = Vec::with_capacity(DEFAULT_CAPACITY);
        value
            .into_par_iter()
            .unzip_into_vecs(&mut keys_source, &mut values);

        let keys = KdeezNuts::new_from_slice_parallel(&keys_source)?;
        assert_eq!(
            keys.size(),
            values.len(),
            "`keys` and `values` should always have the same length"
        );

        Ok(Self {
            keys,
            values: values.into_boxed_slice(),
        })
    }

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
