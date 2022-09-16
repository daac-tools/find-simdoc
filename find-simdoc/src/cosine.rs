//! Searcher for all-pair similar documents in the Cosine space.
use crate::errors::{FindSimdocError, Result};
use crate::feature::{FeatureConfig, FeatureExtractor};
use crate::tfidf::{Idf, Tf};

use all_pairs_hamming::chunked_join::ChunkedJoiner;
use lsh::simhash::SimHasher;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;

/// Searcher for all-pair similar documents in the Cosine space.
///
/// # Approach
///
/// The search steps consist of
///
/// 1. Extracts features from documents,
///    where a feature is a tfidf-weighted vector representation of character or word ngrams.
/// 2. Convert the features into binary sketches through the [simplified simhash](https://dl.acm.org/doi/10.1145/1242572.1242592).
/// 3. Search for similar sketches in the Hamming space using [`ChunkedJoiner`].
///
/// # Examples
///
/// ```
/// use find_simdoc::tfidf::{Idf, Tf};
/// use find_simdoc::CosineSearcher;
///
/// let documents = vec![
///     "Welcome to Jimbocho, the town of books and curry!",
///     "Welcome to Jimbocho, the city of books and curry!",
///     "We welcome you to Jimbocho, the town of books and curry.",
///     "Welcome to the town of books and curry, Jimbocho!",
/// ];
///
/// // Creates a searcher for word unigrams (with random seed value 42).
/// let searcher = CosineSearcher::new(1, Some(' '), Some(42)).unwrap();
/// // Creates a term frequency (TF) weighter.
/// let tf = Tf::new();
/// // Creates a inverse document frequency (IDF) weighter.
/// let idf = Idf::new()
///     .build(documents.iter().clone(), searcher.config())
///     .unwrap();
/// // Builds the database of binary sketches converted from input documents,
/// let searcher = searcher
///     // with the TF weighter and
///     .tf(Some(tf))
///     // the IDF weighter,
///     .idf(Some(idf))
///     // where binary sketches are in the Hamming space of 10*64 dimensions.
///     .build_sketches_in_parallel(documents.iter(), 10)
///     .unwrap();
///
/// // Searches all similar pairs within radius 0.25.
/// let results = searcher.search_similar_pairs(0.25);
/// // A result consists of the left-side id, the right-side id, and their distance.
/// assert_eq!(results, vec![(0, 1, 0.1671875), (0, 3, 0.246875)]);
/// ```
pub struct CosineSearcher {
    config: FeatureConfig,
    hasher: SimHasher,
    tf: Option<Tf>,
    idf: Option<Idf<u64>>,
    joiner: Option<ChunkedJoiner<u64>>,
    shows_progress: bool,
}

impl CosineSearcher {
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
        let hasher = SimHasher::new(seeder.next_u64());
        Ok(Self {
            config,
            hasher,
            tf: None,
            idf: None,
            joiner: None,
            shows_progress: false,
        })
    }

    /// Shows the progress via the standard error output?
    pub const fn shows_progress(mut self, yes: bool) -> Self {
        self.shows_progress = yes;
        self
    }

    /// Sets the scheme of TF weighting.
    #[allow(clippy::missing_const_for_fn)]
    pub fn tf(mut self, tf: Option<Tf>) -> Self {
        self.tf = tf;
        self
    }

    /// Sets the scheme of IDF weighting.
    #[allow(clippy::missing_const_for_fn)]
    pub fn idf(mut self, idf: Option<Idf<u64>>) -> Self {
        self.idf = idf;
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
            extractor.extract_with_weights(doc, &mut feature);
            if let Some(tf) = self.tf.as_ref() {
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
        // TODO: Show progress
        let mut sketches: Vec<_> = documents
            .into_iter()
            .enumerate()
            .par_bridge()
            .map(|(i, doc)| {
                let doc = doc.as_ref();
                // TODO: Returns the error value (but I dont know the manner).
                assert!(!doc.is_empty(), "Input document must not be empty.");
                let mut feature = vec![];
                extractor.extract_with_weights(doc, &mut feature);
                if let Some(tf) = self.tf.as_ref() {
                    tf.tf(&mut feature);
                }
                if let Some(idf) = self.idf.as_ref() {
                    for (term, weight) in feature.iter_mut() {
                        *weight *= idf.idf(*term);
                    }
                }
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
        self.joiner.as_ref().unwrap().similar_pairs(radius)
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
