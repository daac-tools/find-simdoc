use std::hash::Hash;

use hashbrown::HashSet;

pub fn jaccard_distance<I, T>(lhs: I, rhs: I) -> f64
where
    I: IntoIterator<Item = T>,
    T: Hash + Eq,
{
    let h1 = HashSet::<T>::from_iter(lhs);
    let h2 = HashSet::<T>::from_iter(rhs);
    1. - (h1.intersection(&h2).count() as f64) / (h1.union(&h2).count() as f64)
}
