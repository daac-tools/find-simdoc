pub mod minhash;
pub mod simhash;

use std::hash::Hash;

use hashbrown::HashSet;

#[inline(always)]
pub fn hash_u64(x: u64, seed: u64) -> u64 {
    fasthash::city::hash64_with_seed(x.to_le_bytes(), seed)
}

pub fn jaccard_distance<I, T>(lhs: I, rhs: I) -> f64
where
    I: IntoIterator<Item = T>,
    T: Hash + Eq,
{
    let a = HashSet::<T>::from_iter(lhs);
    let b = HashSet::<T>::from_iter(rhs);
    1. - (a.intersection(&b).count() as f64) / (a.union(&b).count() as f64)
}
