use std::hash::Hash;

use hashbrown::{HashMap, HashSet};

pub fn jaccard_distance<I, T>(lhs: I, rhs: I) -> f64
where
    I: IntoIterator<Item = T>,
    T: Hash + Eq,
{
    let a = HashSet::<T>::from_iter(lhs);
    let b = HashSet::<T>::from_iter(rhs);
    1. - (a.intersection(&b).count() as f64) / (a.union(&b).count() as f64)
}

pub fn cosine_distance<I, T>(lhs: I, rhs: I) -> f64
where
    I: IntoIterator<Item = (T, f64)>,
    T: Hash + Eq,
{
    let a = HashMap::<T, f64>::from_iter(lhs);
    let b = HashMap::<T, f64>::from_iter(rhs);
    let norms = norm(&a) * norm(&b);
    1. - if norms > 0. {
        dot(&a, &b) / (norm(&a) * norm(&b))
    } else {
        0.
    }
}

fn norm<T>(data: &HashMap<T, f64>) -> f64
where
    T: Hash + Eq,
{
    dot(data, data).sqrt()
}

fn dot<T>(lhs: &HashMap<T, f64>, rhs: &HashMap<T, f64>) -> f64
where
    T: Hash + Eq,
{
    lhs.iter()
        .map(|(key, val_a)| match rhs.get(key) {
            Some(val_b) => val_a * val_b,
            None => 0.,
        })
        .sum()
}
