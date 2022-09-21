//! A fast and compact implementation of similarity self-join on binary sketches in the Hamming space.
use hashbrown::HashSet;

use crate::errors::{AllPairsHammingError, Result};
use crate::multi_sort::MultiSort;
use crate::sketch::Sketch;

/// A fast and compact implementation of similarity self-join on binary sketches in the Hamming space.
/// The algorithm employs a modified variant of the sketch sorting with the multi-index approach.
///
/// # Complexities
///
/// The time and memory complexities are linear in the input and output size.
///
/// # Examples
///
/// ```
/// use all_pairs_hamming::ChunkedJoiner;
///
/// let mut joiner = ChunkedJoiner::<u8>::new(2);
/// joiner.add([0b1111, 0b1001]);
/// joiner.add([0b1101, 0b1001]);
/// joiner.add([0b0101, 0b0001]);
///
/// let mut results = joiner.similar_pairs(0.15);
/// assert_eq!(results, vec![(0, 1, 0.0625), (1, 2, 0.125)]);
/// ```
///
/// # References
///
/// - Tabei, Uno, Sugiyama, and Tsuda.
///   [Single versus Multiple Sorting in All Pairs Similarity Search](https://proceedings.mlr.press/v13/tabei10a.html).
///   ACML, 2010
/// - J. Qin et al.
///   [Generalizing the Pigeonhole Principle for Similarity Search in Hamming Space](https://doi.org/10.1109/TKDE.2019.2899597).
///   IEEE Transactions on Knowledge and Data Engineering, 2021
pub struct ChunkedJoiner<S> {
    chunks: Vec<Vec<S>>,
    shows_progress: bool,
}

impl<S> ChunkedJoiner<S>
where
    S: Sketch,
{
    /// Creates an instance, handling sketches of `num_chunks` chunks, i.e.,
    /// in `S::dim() * num_chunks` dimensions.
    pub fn new(num_chunks: usize) -> Self {
        Self {
            chunks: vec![vec![]; num_chunks],
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
        let num_chunks = self.num_chunks();
        let mut iter = sketch.into_iter();
        for chunk in self.chunks.iter_mut() {
            chunk.push(iter.next().ok_or_else(|| {
                let msg = format!("The input sketch must include {num_chunks} chunks at least.");
                AllPairsHammingError::input(msg)
            })?);
        }
        Ok(())
    }

    /// Finds all similar pairs whose normalized Hamming distance is within `radius`,
    /// returning triplets of the left-side id, the right-side id, and thier distance.
    pub fn similar_pairs(&self, radius: f64) -> Vec<(usize, usize, f64)> {
        let dimension = S::dim() * self.num_chunks();
        let hamradius = (dimension as f64 * radius).ceil() as usize;
        if self.shows_progress {
            eprintln!(
                "[ChunkedJoiner::similar_pairs] #dimensions={dimension}, hamradius={hamradius}"
            );
        }

        // TODO: Threading.
        let mut candidates = HashSet::new();
        for (j, chunk) in self.chunks.iter().enumerate() {
            // Based on the general pigeonhole principle.
            // https://doi.org/10.1109/TKDE.2019.2899597
            if j + hamradius + 1 < self.chunks.len() {
                continue;
            }
            let r = (j + hamradius + 1 - self.chunks.len()) / self.chunks.len();
            MultiSort::new().similar_pairs(chunk, r, &mut candidates);

            if self.shows_progress {
                eprintln!(
                    "[ChunkedJoiner::similar_pairs] Processed {}/{}...",
                    j + 1,
                    self.chunks.len()
                );
                eprintln!(
                    "[ChunkedJoiner::similar_pairs] #candidates={}",
                    candidates.len()
                );
            }
        }
        if self.shows_progress {
            eprintln!("[ChunkedJoiner::similar_pairs] Done");
        }

        let mut candidates: Vec<_> = candidates.into_iter().collect();
        candidates.sort_unstable();

        let bound = (dimension as f64 * radius) as usize;
        let mut matched = vec![];

        for (i, j) in candidates {
            if let Some(dist) = self.hamming_distance(i, j, bound) {
                let dist = dist as f64 / dimension as f64;
                if dist <= radius {
                    matched.push((i, j, dist));
                }
            }
        }
        if self.shows_progress {
            eprintln!("[ChunkedJoiner::similar_pairs] #matched={}", matched.len());
        }
        matched
    }

    /// Gets the number of chunks.
    pub fn num_chunks(&self) -> usize {
        self.chunks.len()
    }

    /// Gets the number of stored sketches.
    pub fn num_sketches(&self) -> usize {
        self.chunks.get(0).map(|v| v.len()).unwrap_or(0)
    }

    /// Gets the memory usage in bytes.
    pub fn memory_in_bytes(&self) -> usize {
        self.num_chunks() * self.num_sketches() * std::mem::size_of::<S>()
    }

    fn hamming_distance(&self, i: usize, j: usize, bound: usize) -> Option<usize> {
        let mut dist = 0;
        for chunk in &self.chunks {
            dist += chunk[i].hamdist(chunk[j]);
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

        let mut joiner = ChunkedJoiner::new(2);
        for s in sketches {
            joiner.add([(s & 0xFF) as u8, (s >> 8) as u8]).unwrap();
        }
        let mut results = joiner.similar_pairs(radius);
        results.sort_by_key(|&(i, j, _)| (i, j));
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
        let mut joiner = ChunkedJoiner::new(2);
        let result = joiner.add([0u64]);
        assert!(result.is_err());
    }
}
