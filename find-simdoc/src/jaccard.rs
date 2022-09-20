//! Searcher for all-pair similar documents in the Jaccard space.
use std::sync::Mutex;

use crate::errors::{FindSimdocError, Result};
use crate::feature::{FeatureConfig, FeatureExtractor};

use all_pairs_hamming::chunked_join::ChunkedJoiner;
use lsh::minhash::MinHasher;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;

/// Searcher for all-pair similar documents in the Jaccard space.
///
/// # Approach
///
/// The search steps consist of
///
/// 1. Extracts features from documents,
///    where a feature is a set representation of character or word ngrams.
/// 2. Convert the features into binary sketches through the [1-bit minwise hashing](https://dl.acm.org/doi/abs/10.1145/1772690.1772759).
/// 3. Search for similar sketches in the Hamming space using [`ChunkedJoiner`].
///
/// # Examples
///
/// ```
/// use find_simdoc::JaccardSearcher;
///
/// let documents = vec![
///     "Welcome to Jimbocho, the town of books and curry!",
///     "Welcome to Jimbocho, the city of books and curry!",
///     "We welcome you to Jimbocho, the town of books and curry.",
///     "Welcome to the town of books and curry, Jimbocho!",
/// ];
///
/// // Creates a searcher for character trigrams (with random seed value 42).
/// let searcher = JaccardSearcher::new(3, None, Some(42))
///     .unwrap()
///     // Builds the database of binary sketches converted from input documents,
///     // where binary sketches are in the Hamming space of 20*64 dimensions.
///     .build_sketches_in_parallel(documents.iter(), 20)
///     .unwrap();
///
/// // Searches all similar pairs within radius 0.25.
/// let results = searcher.search_similar_pairs(0.25);
/// assert_eq!(results, vec![(0, 1, 0.19375), (0, 2, 0.2125), (0, 3, 0.2328125)]);
/// ```
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
        let seed = seed.unwrap_or_else(rand::random::<u64>);
        let mut seeder = rand_xoshiro::SplitMix64::seed_from_u64(seed);
        let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64())?;
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
        let extractor = FeatureExtractor::new(&self.config);

        let mut feature = vec![];
        for (i, doc) in documents.into_iter().enumerate() {
            if self.shows_progress && (i + 1) % 10000 == 0 {
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

    /// Builds the database of sketches from input documents in parallel.
    ///
    /// # Arguments
    ///
    /// * `documents` - List of documents (must not include an empty string).
    /// * `num_chunks` - Number of chunks of sketches, indicating that
    ///                  the number of dimensions in the Hamming space is `num_chunks*64`.
    ///
    /// # Notes
    ///
    /// The progress is not printed even if `shows_progress = true`.
    pub fn build_sketches_in_parallel<I, D>(
        mut self,
        documents: I,
        num_chunks: usize,
    ) -> Result<Self>
    where
        I: Iterator<Item = D> + Send,
        D: AsRef<str> + Send,
    {
        let extractor = FeatureExtractor::new(&self.config);
        #[allow(clippy::mutex_atomic)]
        let processed = Mutex::new(0usize);
        let mut sketches: Vec<_> = documents
            .into_iter()
            .enumerate()
            .par_bridge()
            .map(|(i, doc)| {
                #[allow(clippy::mutex_atomic)]
                {
                    // Mutex::lock also locks eprintln.
                    let mut cnt = processed.lock().unwrap();
                    *cnt += 1;
                    if self.shows_progress && *cnt % 10000 == 0 {
                        eprintln!("Processed {} documents...", *cnt);
                    }
                }
                let doc = doc.as_ref();
                // TODO: Returns the error value (but I dont know the manner).
                assert!(!doc.is_empty(), "Input document must not be empty.");
                let mut feature = vec![];
                extractor.extract(doc, &mut feature);
                let mut gen = self.hasher.iter(&feature);
                let sketch: Vec<_> = (0..num_chunks).map(|_| gen.next().unwrap()).collect();
                (i, sketch)
            })
            .collect();
        sketches.par_sort_by_key(|&(i, _)| i);

        let mut joiner = ChunkedJoiner::<u64>::new(num_chunks).shows_progress(self.shows_progress);
        for (_, sketch) in sketches {
            joiner.add(sketch).unwrap();
        }
        self.joiner = Some(joiner);
        Ok(self)
    }

    /// Searches for all pairs of similar documents within an input radius, returning
    /// triplets of the left-side id, the right-side id, and their distance.
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
    pub const fn config(&self) -> &FeatureConfig {
        &self.config
    }
}
