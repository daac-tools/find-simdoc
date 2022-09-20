//! Simplified simhash for the Cosine similarity.
use rand_xoshiro::rand_core::{RngCore, SeedableRng};

/// [Simplified simhash](https://dl.acm.org/doi/10.1145/2063576.2063737) for Cosine similarity.
pub struct SimHasher {
    seed: u64,
}

impl SimHasher {
    /// Creates an instance.
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Creates an iterator to generate sketches from an input feature.
    pub fn iter<'a>(&self, feature: &'a [(u64, f64)]) -> SimHashIter<'a> {
        SimHashIter {
            feature,
            seeder: rand_xoshiro::SplitMix64::seed_from_u64(self.seed),
            weights: [0.; 64],
        }
    }
}

/// Iterator to generate sketches with the simplified simhash.
pub struct SimHashIter<'a> {
    feature: &'a [(u64, f64)],
    seeder: rand_xoshiro::SplitMix64,
    weights: [f64; 64],
}

impl<'a> Iterator for SimHashIter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.weights.fill(0.);
        let seed = self.seeder.next_u64();
        for (h, x) in self
            .feature
            .iter()
            .map(|&(i, x)| (crate::lsh::hash_u64(i, seed), x))
        {
            for (j, w) in self.weights.iter_mut().enumerate() {
                if (h >> j) & 1 == 0 {
                    *w += x;
                } else {
                    *w -= x;
                }
            }
        }
        Some(
            self.weights
                .iter()
                .fold(0, |acc, w| if *w >= 0. { (acc << 1) | 1 } else { acc << 1 }),
        )
    }
}
