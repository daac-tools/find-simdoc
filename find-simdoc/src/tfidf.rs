//! Weighters of TF-IDF.
use std::hash::Hash;

use hashbrown::{HashMap, HashSet};

use crate::errors::{FindSimdocError, Result};
use crate::feature::{FeatureConfig, FeatureExtractor};

/// Weighter of inverse document frequency.
#[derive(Default)]
pub struct Idf<T> {
    counter: HashMap<T, usize>,
    dedup: HashSet<T>,
    num_docs: usize,
    smooth: bool,
}

impl<T> Idf<T>
where
    T: Hash + Eq + Copy + Default,
{
    /// Creates an instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables smoothing.
    pub const fn smooth(mut self, yes: bool) -> Self {
        self.smooth = yes;
        self
    }

    /// Trains the frequency of terms for a document.
    pub fn add(&mut self, terms: &[T]) {
        self.dedup.clear();
        for &term in terms {
            if self.dedup.insert(term) {
                self.counter
                    .entry(term)
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        }
        self.num_docs += 1;
    }

    /// Gets the number of input documents.
    pub const fn num_docs(&self) -> usize {
        self.num_docs
    }

    /// Computes the IDF of an input term.
    pub fn idf(&self, term: T) -> f64 {
        let c = usize::from(self.smooth);
        let n = (self.num_docs + c) as f64;
        let m = (*self.counter.get(&term).unwrap() + c) as f64;
        (n / m).log10() + 1.
    }
}

impl Idf<u64> {
    /// Trains the term frequency of input documents.
    ///
    /// # Arguments
    ///
    /// * `documents` - List of documents.
    /// * `config` - Configuration of feature extraction. Use the same configuration as that in search.
    pub fn build<I, D>(mut self, documents: I, config: FeatureConfig) -> Result<Self>
    where
        I: IntoIterator<Item = D>,
        D: AsRef<str>,
    {
        let extractor = FeatureExtractor::new(config);
        let mut feature = vec![];
        for doc in documents {
            let doc = doc.as_ref();
            if doc.is_empty() {
                return Err(FindSimdocError::input("Input document must not be empty."));
            }
            extractor.extract(doc, &mut feature);
            self.add(&feature);
        }
        Ok(self)
    }
}

/// Weighter of term frequency.
#[derive(Default)]
pub struct Tf {
    sublinear: bool,
}

impl Tf {
    /// Creates an instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables sublinear normalization.
    pub const fn sublinear(mut self, yes: bool) -> Self {
        self.sublinear = yes;
        self
    }

    /// Computes the TF of input terms.
    pub fn tf<T>(&self, terms: &mut [(T, f64)])
    where
        T: Hash + Eq + Copy + Default,
    {
        let counter = self.count(terms);
        let total = terms.len() as f64;
        for (term, weight) in terms {
            let cnt = *counter.get(term).unwrap() as f64;
            *weight = if self.sublinear {
                cnt.log10() + 1.
            } else {
                cnt / total
            };
        }
    }

    fn count<T>(&self, terms: &mut [(T, f64)]) -> HashMap<T, usize>
    where
        T: Hash + Eq + Copy + Default,
    {
        let mut counter = HashMap::new();
        for &(term, _) in terms.iter() {
            counter.entry(term).and_modify(|c| *c += 1).or_insert(1);
        }
        counter
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_idf() {
        let mut idf = Idf::new();
        idf.add(&['A', 'A', 'C']);
        idf.add(&['A', 'C']);
        idf.add(&['B', 'A']);

        assert_eq!(idf.num_docs(), 3);

        idf = idf.smooth(false);
        assert_eq!(idf.idf('A'), (3f64 / 3f64).log10() + 1.);
        assert_eq!(idf.idf('B'), (3f64 / 1f64).log10() + 1.);
        assert_eq!(idf.idf('C'), (3f64 / 2f64).log10() + 1.);

        idf = idf.smooth(true);
        assert_eq!(idf.idf('A'), (4f64 / 4f64).log10() + 1.);
        assert_eq!(idf.idf('B'), (4f64 / 2f64).log10() + 1.);
        assert_eq!(idf.idf('C'), (4f64 / 3f64).log10() + 1.);
    }

    #[test]
    fn test_tf() {
        let mut tf = Tf::new();
        let mut terms = vec![('A', 0.), ('B', 0.), ('A', 0.)];

        tf = tf.sublinear(false);
        tf.tf(&mut terms);
        assert_eq!(
            terms.clone(),
            vec![('A', 2. / 3.), ('B', 1. / 3.), ('A', 2. / 3.)]
        );

        tf = tf.sublinear(true);
        tf.tf(&mut terms);
        assert_eq!(
            terms.clone(),
            vec![
                ('A', 2f64.log10() + 1.),
                ('B', 1f64.log10() + 1.),
                ('A', 2f64.log10() + 1.)
            ]
        );
    }
}
