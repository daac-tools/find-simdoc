use hashbrown::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Default)]
pub struct Idf<T> {
    counter: HashMap<T, usize>,
    dedup: HashSet<T>,
    num_docs: usize,
}

impl<T> Idf<T>
where
    T: Hash + Eq + Copy + Default,
{
    pub fn new() -> Self {
        Self::default()
    }

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

    pub const fn num_docs(&self) -> usize {
        self.num_docs
    }

    pub fn idf(&self, term: T) -> f64 {
        let n = self.num_docs as f64;
        let m = *self.counter.get(&term).unwrap() as f64;
        (n / m).log10() + 1.
    }

    pub fn idf_smooth(&self, term: T) -> f64 {
        let n = (self.num_docs + 1) as f64;
        let m = (*self.counter.get(&term).unwrap() + 1) as f64;
        (n / m).log10() + 1.
    }
}

#[derive(Default)]
pub struct Tf<T> {
    counter: HashMap<T, usize>,
}

impl<T> Tf<T>
where
    T: Hash + Eq + Copy + Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tf(&mut self, terms: &mut [(T, f64)]) {
        self.count(terms);
        let total = terms.len() as f64;
        for (term, weight) in terms {
            let cnt = *self.counter.get(term).unwrap() as f64;
            *weight = cnt / total;
        }
    }

    pub fn tf_sublinear(&mut self, terms: &mut [(T, f64)]) {
        self.count(terms);
        for (term, weight) in terms {
            let cnt = *self.counter.get(term).unwrap() as f64;
            *weight = cnt.log10() + 1.;
        }
    }

    fn count(&mut self, terms: &mut [(T, f64)]) {
        self.counter.clear();
        for &(term, _) in terms.iter() {
            self.counter
                .entry(term)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
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

        assert_eq!(idf.idf('A'), (3f64 / 3f64).log10() + 1.);
        assert_eq!(idf.idf('B'), (3f64 / 1f64).log10() + 1.);
        assert_eq!(idf.idf('C'), (3f64 / 2f64).log10() + 1.);

        assert_eq!(idf.idf_smooth('A'), (4f64 / 4f64).log10() + 1.);
        assert_eq!(idf.idf_smooth('B'), (4f64 / 2f64).log10() + 1.);
        assert_eq!(idf.idf_smooth('C'), (4f64 / 3f64).log10() + 1.);
    }

    #[test]
    fn test_tf() {
        let mut tf = Tf::new();
        let mut terms = vec![('A', 0.), ('B', 0.), ('A', 0.)];
        tf.tf(&mut terms);
        assert_eq!(
            terms.clone(),
            vec![('A', 2. / 3.), ('B', 1. / 3.), ('A', 2. / 3.)]
        );
        tf.tf_sublinear(&mut terms);
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
