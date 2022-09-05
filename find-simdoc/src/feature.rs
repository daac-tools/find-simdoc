use std::hash::{Hash, Hasher};
use std::ops::Range;

use fasthash::{CityHasher, FastHasher};

use crate::shingling::ShingleIter;

#[derive(Clone, Copy, Debug)]
pub struct FeatureConfig {
    window_size: usize,
    delimiter: Option<char>,
    seed: u64,
}

impl FeatureConfig {
    pub fn new(window_size: usize, delimiter: Option<char>, seed: u64) -> Self {
        assert!(window_size >= 1);
        Self {
            window_size,
            delimiter,
            seed,
        }
    }

    fn hash<I, T>(&self, iter: I) -> u64
    where
        I: IntoIterator<Item = T>,
        T: Hash,
    {
        let mut s = CityHasher::with_seed(self.seed);
        for t in iter {
            t.hash(&mut s);
        }
        s.finish()
    }
}

pub struct FeatureExtractor {
    config: FeatureConfig,
    token_ranges: Vec<Range<usize>>,
}

impl FeatureExtractor {
    pub const fn new(config: FeatureConfig) -> Self {
        Self {
            config,
            token_ranges: vec![],
        }
    }

    pub fn extract<S>(&mut self, text: S, features: &mut Vec<u64>)
    where
        S: AsRef<str>,
    {
        let text = text.as_ref();

        features.clear();
        if self.config.delimiter.is_none() && self.config.window_size == 1 {
            // The simplest case.
            text.chars().for_each(|c| features.push(c as u64));
        } else {
            self.tokenize(text);
            for ranges in ShingleIter::new(&self.token_ranges, self.config.window_size) {
                features.push(self.config.hash(ranges.iter().cloned().map(|r| &text[r])));
            }
        }
    }

    pub fn extract_with_weights<S>(&mut self, text: S, features: &mut Vec<(u64, f64)>)
    where
        S: AsRef<str>,
    {
        let text = text.as_ref();

        features.clear();
        if self.config.delimiter.is_none() && self.config.window_size == 1 {
            // The simplest case.
            text.chars().for_each(|c| {
                let f = c as u64;
                let w = 1.;
                features.push((f, w))
            });
        } else {
            self.tokenize(text);
            for ranges in ShingleIter::new(&self.token_ranges, self.config.window_size) {
                let f = self.config.hash(ranges.iter().cloned().map(|r| &text[r]));
                let w = 1.;
                features.push((f, w))
            }
        }
    }

    fn tokenize(&mut self, text: &str) {
        self.token_ranges.clear();

        let mut offset = 0;
        if let Some(delim) = self.config.delimiter {
            while offset < text.len() {
                let len = text[offset..].find(delim);
                if let Some(len) = len {
                    self.token_ranges.push(offset..offset + len);
                    offset += len + 1;
                } else {
                    self.token_ranges.push(offset..text.len());
                    break;
                }
            }
        } else {
            for c in text.chars() {
                let len = c.len_utf8();
                self.token_ranges.push(offset..offset + len);
                offset += len;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_unigram() {
        let config = FeatureConfig::new(1, None, 42);
        let mut extractor = FeatureExtractor::new(config);

        let text = "abcd";
        let mut features = vec![];

        extractor.extract(text, &mut features);
        assert_eq!(
            features,
            vec!['a' as u64, 'b' as u64, 'c' as u64, 'd' as u64]
        )
    }

    #[test]
    fn test_char_bigram() {
        let config = FeatureConfig::new(2, None, 42);
        let mut extractor = FeatureExtractor::new(config);

        let text = "abcd";
        let mut features = vec![];

        extractor.extract(text, &mut features);
        assert_eq!(
            features,
            vec![
                config.hash(&["a", "b"]),
                config.hash(&["b", "c"]),
                config.hash(&["c", "d"]),
            ]
        )
    }

    #[test]
    fn test_char_trigram() {
        let config = FeatureConfig::new(3, None, 42);
        let mut extractor = FeatureExtractor::new(config);

        let text = "abcd";
        let mut features = vec![];

        extractor.extract(text, &mut features);
        assert_eq!(
            features,
            vec![config.hash(&["a", "b", "c"]), config.hash(&["b", "c", "d"]),]
        )
    }

    #[test]
    fn test_word_unigram() {
        let config = FeatureConfig::new(1, Some(' '), 42);
        let mut extractor = FeatureExtractor::new(config);

        let text = "abc de fgh";
        let mut features = vec![];

        extractor.extract(text, &mut features);
        assert_eq!(
            features,
            vec![
                config.hash(&["abc"]),
                config.hash(&["de"]),
                config.hash(&["fgh"]),
            ]
        )
    }

    #[test]
    fn test_word_bigram() {
        let config = FeatureConfig::new(2, Some(' '), 42);
        let mut extractor = FeatureExtractor::new(config);

        let text = "abc de fgh";
        let mut features = vec![];

        extractor.extract(text, &mut features);
        assert_eq!(
            features,
            vec![config.hash(&["abc", "de"]), config.hash(&["de", "fgh"]),]
        )
    }

    #[test]
    fn test_word_trigram() {
        let config = FeatureConfig::new(3, Some(' '), 42);
        let mut extractor = FeatureExtractor::new(config);

        let text = "abc de fgh";
        let mut features = vec![];

        extractor.extract(text, &mut features);
        assert_eq!(features, vec![config.hash(&["abc", "de", "fgh"])])
    }
}
