//! Locality-sensitive hashings.
pub mod minhash;
pub mod simhash;

use std::hash::Hash;

use hashbrown::HashSet;
use rand_xoshiro::rand_core::{RngCore, SeedableRng};

/// Generates a hash value.
#[inline(always)]
pub(crate) fn hash_u64(x: u64, seed: u64) -> u64 {
    rand_xoshiro::SplitMix64::seed_from_u64(x ^ seed).next_u64()
}

/// Computes the Jaccard distance.
///
/// # Examples
///
/// ```
/// use find_simdoc::lsh::jaccard_distance;
///
/// let x = vec![1, 2, 4];
/// let y = vec![1, 2, 5, 7];
/// assert_eq!(jaccard_distance(x, y), 0.6);
/// ```
pub fn jaccard_distance<I, T>(lhs: I, rhs: I) -> f64
where
    I: IntoIterator<Item = T>,
    T: Hash + Eq,
{
    let a = HashSet::<T>::from_iter(lhs);
    let b = HashSet::<T>::from_iter(rhs);
    1. - (a.intersection(&b).count() as f64) / (a.union(&b).count() as f64)
}
