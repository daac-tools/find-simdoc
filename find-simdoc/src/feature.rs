//! Feature extractor.
use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::Range;

use ahash::RandomState;
use rand::{RngCore, SeedableRng};

use crate::errors::{FindSimdocError, Result};
use crate::shingling::ShingleIter;

/// Configuration of feature extraction.
#[derive(Clone, Debug)]
pub struct FeatureConfig {
    window_size: usize,
    delimiter: Option<char>,
    build_hasher: RandomState,
}

impl FeatureConfig {
    /// Creates an instance.
    ///
    /// # Arguments
    ///
    /// * `window_size` - Window size for w-shingling in feature extraction (must be more than 0).
    /// * `delimiter` - Delimiter for recognizing words as tokens in feature extraction.
    ///                 If `None`, characters are used for tokens.
    /// * `seed` - Seed value for random values.
    pub fn new(window_size: usize, delimiter: Option<char>, seed: u64) -> Result<Self> {
        if window_size == 0 {
            return Err(FindSimdocError::input("Window size must not be 0."));
        }
        let mut seeder = rand_xoshiro::SplitMix64::seed_from_u64(seed);
        let build_hasher = RandomState::with_seeds(
            seeder.next_u64(),
            seeder.next_u64(),
            seeder.next_u64(),
            seeder.next_u64(),
        );
        Ok(Self {
            window_size,
            delimiter,
            build_hasher,
        })
    }

    fn hash<I, T>(&self, iter: I) -> u64
    where
        I: IntoIterator<Item = T>,
        T: Hash,
    {
        let mut s = self.build_hasher.build_hasher();
        for t in iter {
            t.hash(&mut s);
        }
        s.finish()
    }
}

/// Extractor of feature vectors.
pub struct FeatureExtractor<'a> {
    config: &'a FeatureConfig,
}

impl<'a> FeatureExtractor<'a> {
    /// Creates an instance.
    pub const fn new(config: &'a FeatureConfig) -> Self {
        Self { config }
    }

    /// Extracts a feature vector from an input text.
    pub fn extract<S>(&self, text: S, feature: &mut Vec<u64>)
    where
        S: AsRef<str>,
    {
        let text = text.as_ref();

        feature.clear();
        if self.config.delimiter.is_none() && self.config.window_size == 1 {
            // The simplest case.
            text.chars().for_each(|c| feature.push(c as u64));
        } else {
            let token_ranges = self.tokenize(text);
            for ranges in ShingleIter::new(&token_ranges, self.config.window_size) {
                feature.push(self.config.hash(ranges.iter().cloned().map(|r| &text[r])));
            }
        }
    }

    /// Extracts a feature vector from an input text with weights of 1.0.
    pub fn extract_with_weights<S>(&self, text: S, feature: &mut Vec<(u64, f64)>)
    where
        S: AsRef<str>,
    {
        let text = text.as_ref();

        feature.clear();
        if self.config.delimiter.is_none() && self.config.window_size == 1 {
            // The simplest case.
            text.chars().for_each(|c| {
                let f = c as u64;
                let w = 1.;
                feature.push((f, w))
            });
        } else {
            let token_ranges = self.tokenize(text);
            for ranges in ShingleIter::new(&token_ranges, self.config.window_size) {
                let f = self.config.hash(ranges.iter().cloned().map(|r| &text[r]));
                let w = 1.;
                feature.push((f, w))
            }
        }
    }

    fn tokenize(&self, text: &str) -> Vec<Range<usize>> {
        let mut token_ranges = vec![];
        for _ in 1..self.config.window_size {
            token_ranges.push(0..0); // BOS
        }
        let mut offset = 0;
        if let Some(delim) = self.config.delimiter {
            while offset < text.len() {
                let len = text[offset..].find(delim);
                if let Some(len) = len {
                    token_ranges.push(offset..offset + len);
                    offset += len + 1;
                } else {
                    token_ranges.push(offset..text.len());
                    break;
                }
            }
        } else {
            for c in text.chars() {
                let len = c.len_utf8();
                token_ranges.push(offset..offset + len);
                offset += len;
            }
        }
        for _ in 1..self.config.window_size {
            token_ranges.push(text.len()..text.len()); // EOS
        }
        token_ranges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_unigram() {
        let config = FeatureConfig::new(1, None, 42).unwrap();
        let extractor = FeatureExtractor::new(&config);

        let text = "abcd";
        let mut feature = vec![];

        extractor.extract(text, &mut feature);
        assert_eq!(
            feature,
            vec!['a' as u64, 'b' as u64, 'c' as u64, 'd' as u64]
        )
    }

    #[test]
    fn test_char_bigram() {
        let config = FeatureConfig::new(2, None, 42).unwrap();
        let extractor = FeatureExtractor::new(&config);

        let text = "abcd";
        let mut feature = vec![];

        extractor.extract(text, &mut feature);
        assert_eq!(
            feature,
            vec![
                config.hash(&["", "a"]),
                config.hash(&["a", "b"]),
                config.hash(&["b", "c"]),
                config.hash(&["c", "d"]),
                config.hash(&["d", ""]),
            ]
        )
    }

    #[test]
    fn test_char_trigram() {
        let config = FeatureConfig::new(3, None, 42).unwrap();
        let extractor = FeatureExtractor::new(&config);

        let text = "abcd";
        let mut feature = vec![];

        extractor.extract(text, &mut feature);
        assert_eq!(
            feature,
            vec![
                config.hash(&["", "", "a"]),
                config.hash(&["", "a", "b"]),
                config.hash(&["a", "b", "c"]),
                config.hash(&["b", "c", "d"]),
                config.hash(&["c", "d", ""]),
                config.hash(&["d", "", ""]),
            ]
        )
    }

    #[test]
    fn test_word_unigram() {
        let config = FeatureConfig::new(1, Some(' '), 42).unwrap();
        let extractor = FeatureExtractor::new(&config);

        let text = "abc de fgh";
        let mut feature = vec![];

        extractor.extract(text, &mut feature);
        assert_eq!(
            feature,
            vec![
                config.hash(&["abc"]),
                config.hash(&["de"]),
                config.hash(&["fgh"]),
            ]
        )
    }

    #[test]
    fn test_word_bigram() {
        let config = FeatureConfig::new(2, Some(' '), 42).unwrap();
        let extractor = FeatureExtractor::new(&config);

        let text = "abc de fgh";
        let mut feature = vec![];

        extractor.extract(text, &mut feature);
        assert_eq!(
            feature,
            vec![
                config.hash(&["", "abc"]),
                config.hash(&["abc", "de"]),
                config.hash(&["de", "fgh"]),
                config.hash(&["fgh", ""]),
            ]
        )
    }

    #[test]
    fn test_word_trigram() {
        let config = FeatureConfig::new(3, Some(' '), 42).unwrap();
        let extractor = FeatureExtractor::new(&config);

        let text = "abc de fgh";
        let mut feature = vec![];

        extractor.extract(text, &mut feature);
        assert_eq!(
            feature,
            vec![
                config.hash(&["", "", "abc"]),
                config.hash(&["", "abc", "de"]),
                config.hash(&["abc", "de", "fgh"]),
                config.hash(&["de", "fgh", ""]),
                config.hash(&["fgh", "", ""]),
            ]
        )
    }
}
