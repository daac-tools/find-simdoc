//! An exhaustive approach of all-pair similarity search on binary sketches.
use anyhow::{anyhow, Result};

use crate::sketch::Sketch;

/// An exhaustive approach of all-pair similarity search on binary sketches.
pub struct SimpleJoiner<S> {
    sketches: Vec<Vec<S>>,
    num_chunks: usize,
    shows_progress: bool,
}

impl<S> SimpleJoiner<S>
where
    S: Sketch,
{
    /// Creates an instance, handling sketches of `num_chunks` chunks, i.e.,
    /// in `S::dim() * num_chunks` dimensions.
    pub const fn new(num_chunks: usize) -> Self {
        Self {
            sketches: vec![],
            num_chunks,
            shows_progress: false,
        }
    }

    /// Prints the progress with stderr?
    pub const fn shows_progress(mut self, yes: bool) -> Self {
        self.shows_progress = yes;
        self
    }

    /// Appends a sketch of [`Self::num_chunks()`] chunks.
    /// The first [`Self::num_chunks()`] elements of an input iterator is stored.
    /// If the iterator is consumed until obtaining the elements, an error is returned.
    pub fn add<I>(&mut self, sketch: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
    {
        let mut iter = sketch.into_iter();
        let mut sketch = Vec::with_capacity(self.num_chunks());
        for _ in 0..self.num_chunks() {
            sketch.push(iter.next().ok_or_else(|| {
                anyhow!(
                    "The input sketch must include {} chunks at least.",
                    self.num_chunks()
                )
            })?)
        }
        self.sketches.push(sketch);
        Ok(())
    }

    /// Finds all similar pairs whose normalized Hamming distance is within `radius`,
    /// returning triplets of the left-side id, the right-side id, and thier distance.
    pub fn similar_pairs(&self, radius: f64) -> Vec<(usize, usize, f64)> {
        let dimension = S::dim() * self.num_chunks();
        if self.shows_progress {
            eprintln!("[SimpleJoiner::similar_pairs] #dimensions={dimension}");
        }

        let bound = (dimension as f64 * radius) as usize;
        let mut matched = vec![];

        for i in 0..self.sketches.len() {
            if self.shows_progress && (i + 1) % 10000 == 0 {
                eprintln!(
                    "[SimpleJoiner::similar_pairs] Processed {}/{}...",
                    i + 1,
                    self.sketches.len()
                );
            }
            for j in i + 1..self.sketches.len() {
                if let Some(dist) = self.hamming_distance(i, j, bound) {
                    let dist = dist as f64 / dimension as f64;
                    if dist <= radius {
                        matched.push((i, j, dist));
                    }
                }
            }
        }
        if self.shows_progress {
            eprintln!("[SimpleJoiner::similar_pairs] Done");
            eprintln!("[SimpleJoiner::similar_pairs] #matched={}", matched.len());
        }
        matched
    }

    /// Gets the number of chunks.
    pub const fn num_chunks(&self) -> usize {
        self.num_chunks
    }

    /// Gets the number of stored sketches.
    pub fn num_sketches(&self) -> usize {
        self.sketches.len()
    }

    /// Gets the memory usage in bytes.
    pub fn memory_in_bytes(&self) -> usize {
        self.num_chunks() * self.num_sketches() * std::mem::size_of::<S>()
    }

    fn hamming_distance(&self, i: usize, j: usize, bound: usize) -> Option<usize> {
        let xs = &self.sketches[i];
        let ys = &self.sketches[j];
        let mut dist = 0;
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            dist += x.hamdist(y);
            if bound < dist {
                return None;
            }
        }
        Some(dist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_sketches() -> Vec<u16> {
        vec![
            0b_1110_0011_1111_1011, // 0
            0b_0001_0111_0111_1101, // 1
            0b_1100_1101_1000_1100, // 2
            0b_1100_1101_0001_0100, // 3
            0b_1010_1110_0010_1010, // 4
            0b_0111_1001_0011_1111, // 5
            0b_1110_0011_0001_0000, // 6
            0b_1000_0111_1001_0101, // 7
            0b_1110_1101_1000_1101, // 8
            0b_0111_1001_0011_1001, // 9
        ]
    }

    fn naive_search(sketches: &[u16], radius: f64) -> Vec<(usize, usize, f64)> {
        let mut results = vec![];
        for i in 0..sketches.len() {
            let x = sketches[i];
            for j in i + 1..sketches.len() {
                let y = sketches[j];
                let dist = x.hamdist(y);
                let dist = dist as f64 / 16.;
                if dist <= radius {
                    results.push((i, j, dist));
                }
            }
        }
        results
    }

    fn test_similar_pairs(radius: f64) {
        let sketches = example_sketches();
        let expected = naive_search(&sketches, radius);

        let mut joiner = SimpleJoiner::new(2);
        for s in sketches {
            joiner.add([(s & 0xFF) as u8, (s >> 8) as u8]).unwrap();
        }
        let results = joiner.similar_pairs(radius);
        assert_eq!(results, expected);
    }

    #[test]
    fn test_similar_pairs_for_all() {
        for radius in 0..=10 {
            test_similar_pairs(radius as f64 / 10.);
        }
    }

    #[test]
    fn test_short_sketch() {
        let mut joiner = SimpleJoiner::new(2);
        let result = joiner.add([0u64]);
        assert!(result.is_err());
    }
}
