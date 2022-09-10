use crate::errors::{FindSimdocError, Result};
use crate::feature::{FeatureConfig, FeatureExtractor};
use crate::tfidf::{Idf, Tf};

use all_pairs_hamming::chunked_join::ChunkedJoiner;
use lsh::simhash::SimHasher;
use rand::{RngCore, SeedableRng};

pub struct CosineSearcher {
    config: FeatureConfig,
    hasher: SimHasher,
    tf: Option<Tf<u64>>,
    idf: Option<Idf<u64>>,
    joiner: Option<ChunkedJoiner<u64>>,
    shows_progress: bool,
}

impl CosineSearcher {
    pub fn new(window_size: usize, delimiter: Option<char>, seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or_else(rand::random::<u64>);
        let mut seeder = rand_xoshiro::SplitMix64::seed_from_u64(seed);
        let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64());
        let hasher = SimHasher::new(seeder.next_u64());
        Self {
            config,
            hasher,
            tf: None,
            idf: None,
            joiner: None,
            shows_progress: false,
        }
    }

    pub const fn shows_progress(mut self, yes: bool) -> Self {
        self.shows_progress = yes;
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn tf(mut self, tf: Option<Tf<u64>>) -> Self {
        self.tf = tf;
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn idf(mut self, idf: Option<Idf<u64>>) -> Self {
        self.idf = idf;
        self
    }

    pub fn build_sketches<I, D>(mut self, documents: I, num_chunks: usize) -> Result<Self>
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
            if doc.is_empty() {
                return Err(FindSimdocError::input("Input document must not be empty."));
            }
            extractor.extract_with_weights(doc, &mut feature);
            if let Some(tf) = self.tf.as_mut() {
                tf.tf(&mut feature);
            }
            if let Some(idf) = self.idf.as_ref() {
                for (term, weight) in feature.iter_mut() {
                    *weight *= idf.idf(*term);
                }
            }
            joiner.add(self.hasher.iter(&feature)).unwrap();
        }
        self.joiner = Some(joiner);
        Ok(self)
    }

    pub fn search_similar_pairs(&self, radius: f64) -> Vec<(usize, usize, f64)> {
        self.joiner.as_ref().unwrap().similar_pairs(radius)
    }

    pub fn len(&self) -> usize {
        self.joiner
            .as_ref()
            .map_or(0, |joiner| joiner.num_sketches())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn memory_in_bytes(&self) -> usize {
        self.joiner
            .as_ref()
            .map_or(0, |joiner| joiner.memory_in_bytes())
    }

    pub const fn config(&self) -> FeatureConfig {
        self.config
    }
}
