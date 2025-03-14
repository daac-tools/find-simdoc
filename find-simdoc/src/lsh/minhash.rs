//! 1-bit minwise hashing for the Jaccard similarity.
use rand_xoshiro::rand_core::{RngCore, SeedableRng};

/// [1-bit minwise hashing](https://dl.acm.org/doi/abs/10.1145/1772690.1772759) for the Jaccard similarity.
pub struct MinHasher {
    seed: u64,
}

impl MinHasher {
    /// Creates an instance.
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Creates an iterator to generate sketches from an input feature.
    pub fn iter<'a>(&self, feature: &'a [u64]) -> MinHashIter<'a> {
        MinHashIter {
            feature,
            seeder: rand_xoshiro::SplitMix64::seed_from_u64(self.seed),
        }
    }
}

/// Iterator to generate sketches with the 1-bit minwise hashing.
pub struct MinHashIter<'a> {
    feature: &'a [u64],
    seeder: rand_xoshiro::SplitMix64,
}

impl Iterator for MinHashIter<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let mut x = 0;
        for _ in 0..64 {
            let seed = self.seeder.next_u64();
            let h = self
                .feature
                .iter()
                .map(|&i| crate::lsh::hash_u64(i, seed))
                .min()
                .unwrap();
            x = (x << 1) | (h & 1);
        }
        Some(x)
    }
}
