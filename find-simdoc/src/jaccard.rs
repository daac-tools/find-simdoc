use crate::errors::{FindSimdocError, Result};
use crate::feature::{FeatureConfig, FeatureExtractor};

use all_pairs_hamming::chunked_join::ChunkedJoiner;
use lsh::minhash::MinHasher;
use rand::{RngCore, SeedableRng};

/// Searcher in Jaccard space using 1-bit minwise hashing.
pub struct JaccardSearcher {
    config: FeatureConfig,
    hasher: MinHasher,
    joiner: Option<ChunkedJoiner<u64>>,
    shows_progress: bool,
}

impl JaccardSearcher {
    /// Creates an instance.
    ///
    /// # Arguments
    ///
    /// * `window_size` - Window size for w-shingling in feature extraction (must be more than 0).
    /// * `delimiter` - Delimiter for recognizing words as tokens in feature extraction.
    ///                 If `None`, characters are used for tokens.
    /// * `seed` - Seed value for random values.
    pub fn new(window_size: usize, delimiter: Option<char>, seed: Option<u64>) -> Result<Self> {
        if window_size == 0 {
            return Err(FindSimdocError::input("Window size must not be 0."));
        }
        let seed = seed.unwrap_or_else(rand::random::<u64>);
        let mut seeder = rand_xoshiro::SplitMix64::seed_from_u64(seed);
        let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64());
        let hasher = MinHasher::new(seeder.next_u64());
        Ok(Self {
            config,
            hasher,
            joiner: None,
            shows_progress: false,
        })
    }

    /// Shows the progress via the standard error output?
    pub const fn shows_progress(mut self, yes: bool) -> Self {
        self.shows_progress = yes;
        self
    }

    /// Builds the database of sketches from input documents.
    ///
    /// # Arguments
    ///
    /// * `documents` - List of documents (must not include an empty string).
    /// * `num_chunks` - Number of chunks of sketches, indicating that
    ///                  the number of dimensions in the Hamming space is `num_chunks*64`.
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
            extractor.extract(doc, &mut feature);
            joiner.add(self.hasher.iter(&feature)).unwrap();
        }
        self.joiner = Some(joiner);
        Ok(self)
    }

    /// Searches for all pairs of similar documents within an input radius, returning
    /// triplets of the left-side index, the right-side index, and its distance.
    pub fn search_similar_pairs(&self, radius: f64) -> Vec<(usize, usize, f64)> {
        self.joiner.as_ref().map_or_else(Vec::new, |joiner| {
            // In 1-bit minhash, the collision probability is multiplied by 2 over the original.
            // Thus, we should search with the half of the actual radius.
            let mut results = joiner.similar_pairs(radius / 2.);
            // Modifies the distances.
            results.iter_mut().for_each(|(_, _, d)| *d *= 2.);
            results
        })
    }

    /// Gets the number of input documents.
    pub fn len(&self) -> usize {
        self.joiner
            .as_ref()
            .map_or(0, |joiner| joiner.num_sketches())
    }

    /// Checks if the database is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the memory usage in bytes.
    pub fn memory_in_bytes(&self) -> usize {
        self.joiner
            .as_ref()
            .map_or(0, |joiner| joiner.memory_in_bytes())
    }

    /// Gets the configure of feature extraction.
    pub const fn config(&self) -> FeatureConfig {
        self.config
    }
}
