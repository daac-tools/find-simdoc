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
    features: Vec<u64>,
}

impl FeatureExtractor {
    pub fn new(config: FeatureConfig) -> Self {
        Self {
            config,
            token_ranges: vec![],
            features: vec![],
        }
    }

    pub fn extract(&mut self, text: &str) -> &[u64] {
        self.features.clear();
        // The simplest case.
        if self.config.delimiter.is_none() && self.config.window_size == 1 {
            text.chars().for_each(|c| self.features.push(c as u64));
            return &self.features;
        }
        self.tokenize(text);
        self.build_features(text);
        &self.features
    }

    fn tokenize(&mut self, text: &str) {
        self.token_ranges.clear();
        if let Some(delim) = self.config.delimiter {
            let mut offset = 0;
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
            let mut offset = 0;
            for c in text.chars() {
                let len = c.len_utf8();
                self.token_ranges.push(offset..offset + len);
                offset += len;
            }
        }
    }

    fn build_features(&mut self, text: &str) {
        for ranges in ShingleIter::new(&self.token_ranges, self.config.window_size) {
            self.features
                .push(self.config.hash(ranges.iter().cloned().map(|r| &text[r])));
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
        let features = extractor.extract(text);
        assert_eq!(features, ['a' as u64, 'b' as u64, 'c' as u64, 'd' as u64])
    }

    #[test]
    fn test_char_bigram() {
        let config = FeatureConfig::new(2, None, 42);
        let mut extractor = FeatureExtractor::new(config);

        let text = "abcd";
        let features = extractor.extract(text);
        assert_eq!(
            features,
            [
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
        let features = extractor.extract(text);
        assert_eq!(
            features,
            [config.hash(&["a", "b", "c"]), config.hash(&["b", "c", "d"]),]
        )
    }

    #[test]
    fn test_word_unigram() {
        let config = FeatureConfig::new(1, Some(' '), 42);
        let mut extractor = FeatureExtractor::new(config);

        let text = "abc de fgh";
        let features = extractor.extract(text);
        assert_eq!(
            features,
            [
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
        let features = extractor.extract(text);
        assert_eq!(
            features,
            [config.hash(&["abc", "de"]), config.hash(&["de", "fgh"]),]
        )
    }

    #[test]
    fn test_word_trigram() {
        let config = FeatureConfig::new(3, Some(' '), 42);
        let mut extractor = FeatureExtractor::new(config);

        let text = "abc de fgh";
        let features = extractor.extract(text);
        assert_eq!(features, [config.hash(&["abc", "de", "fgh"])])
    }
}
