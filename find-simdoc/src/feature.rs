use std::hash::{Hash, Hasher};

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

    fn hash<T>(&self, items: &[T]) -> u64
    where
        T: Hash,
    {
        let mut s = CityHasher::with_seed(self.seed);
        for t in items {
            t.hash(&mut s);
        }
        s.finish()
    }
}

pub struct FeatureExtractor<'a> {
    config: FeatureConfig,
    tokens: Vec<&'a str>,
    features: Vec<u64>,
}

impl<'a> FeatureExtractor<'a> {
    pub fn new(config: FeatureConfig) -> Self {
        Self {
            config,
            tokens: vec![],
            features: vec![],
        }
    }

    pub fn extract(&mut self, text: &'a str) -> &[u64] {
        self.features.clear();
        // The simplest case.
        if self.config.delimiter.is_none() && self.config.window_size == 1 {
            text.chars().for_each(|c| self.features.push(c as u64));
            return &self.features;
        }
        self.tokenize(text);
        self.build_features();
        &self.features
    }

    fn tokenize(&mut self, text: &'a str) {
        self.tokens.clear();
        if let Some(delim) = self.config.delimiter {
            text.split(delim).for_each(|s| self.tokens.push(s));
        } else {
            let mut offset = 0;
            for c in text.chars() {
                let len_utf8 = c.len_utf8();
                self.tokens.push(&text[offset..offset + len_utf8]);
                offset += len_utf8;
            }
        }
    }

    fn build_features(&mut self) {
        for gram in ShingleIter::new(&self.tokens, self.config.window_size) {
            self.features.push(self.config.hash(gram));
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
