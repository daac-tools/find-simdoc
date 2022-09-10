use crate::feature::{FeatureConfig, FeatureExtractor};
use all_pairs_hamming::chunked_join::ChunkedJoiner;
use lsh::minhash::MinHasher;
use rand::{RngCore, SeedableRng};

pub struct JaccardSearcher {
    config: FeatureConfig,
    hasher: MinHasher,
    shows_progress: bool,
    joiner: Option<ChunkedJoiner<u64>>,
}

impl JaccardSearcher {
    pub fn new(window_size: usize, delimiter: Option<char>, seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or_else(rand::random::<u64>);
        let mut seeder = rand_xoshiro::SplitMix64::seed_from_u64(seed);
        let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64());
        let hasher = MinHasher::new(seeder.next_u64());
        Self {
            config,
            hasher,
            shows_progress: false,
            joiner: None,
        }
    }

    pub fn shows_progress(mut self, yes: bool) -> Self {
        self.shows_progress = yes;
        self
    }

    pub fn build_sketches<I, D>(mut self, documents: I, num_chunks: usize) -> Self
    where
        I: IntoIterator<Item = D>,
        D: AsRef<str>,
    {
        let mut joiner = ChunkedJoiner::<u64>::new(num_chunks).shows_progress(self.shows_progress);
        let mut extractor = FeatureExtractor::new(self.config);
        let mut feature = vec![];
        for (i, doc) in documents.into_iter().enumerate() {
            if self.shows_progress && (i + 1) % 1000 == 0 {
                eprintln!("Processed {} documents...", i + 1);
            }
            let doc = doc.as_ref();
            assert!(!doc.is_empty());
            extractor.extract(doc, &mut feature);
            joiner.add(self.hasher.iter(&feature)).unwrap();
        }
        self.joiner = Some(joiner);
        self
    }

    pub fn search_similar_pairs(&self, radius: f64) -> Vec<(usize, usize, f64)> {
        // In 1-bit minhash, the collision probability is multiplied by 2 over the original.
        // Thus, we should search with the half of the actual radius.
        let mut results = self.joiner.as_ref().unwrap().similar_pairs(radius / 2.);
        // Modifies the distances.
        results.iter_mut().for_each(|(_, _, d)| *d *= 2.);
        results
    }

    pub fn len(&self) -> usize {
        if let Some(joiner) = self.joiner.as_ref() {
            joiner.num_sketches()
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn memory_in_bytes(&self) -> usize {
        if let Some(joiner) = self.joiner.as_ref() {
            joiner.memory_in_bytes()
        } else {
            0
        }
    }
}
