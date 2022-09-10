use hashbrown::{HashMap, HashSet};
use std::hash::Hash;

pub struct Idf<T> {
    counter: HashMap<T, usize>,
    dedup: HashSet<T>,
    num_docs: usize,
}

impl<T> Idf<T>
where
    T: Hash + Eq + Copy,
{
    pub fn new() -> Self {
        Self {
            counter: HashMap::new(),
            dedup: HashSet::new(),
            num_docs: 0,
        }
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

    pub fn num_docs(&self) -> usize {
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

pub struct Tf<T> {
    counter: HashMap<T, usize>,
}

impl<T> Tf<T>
where
    T: Hash + Eq + Copy,
{
    pub fn new() -> Self {
        Self {
            counter: HashMap::new(),
        }
    }

    pub fn tf(&mut self, terms: &mut Vec<(T, f64)>) {
        self.count(terms);
        let total = terms.len() as f64;
        for (term, weight) in terms {
            let cnt = *self.counter.get(term).unwrap() as f64;
            *weight = cnt / total;
        }
    }

    pub fn tf_sublinear(&mut self, terms: &mut Vec<(T, f64)>) {
        self.count(terms);
        for (term, weight) in terms {
            let cnt = *self.counter.get(term).unwrap() as f64;
            *weight = cnt.log10() + 1.;
        }
    }

    fn count(&mut self, terms: &mut Vec<(T, f64)>) {
        self.counter.clear();
        for &(term, _) in terms.iter() {
            self.counter
                .entry(term)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
    }
}
