use std::cell::RefCell;
use std::ops::Range;

use crate::bitset64::Bitset64;
use crate::sketch::Sketch;

const SORT_SHIFT: usize = 8;
const SORT_MASK: usize = (1 << SORT_SHIFT) - 1;

#[derive(Clone, Debug, Default)]
struct Record<S> {
    id: usize,
    sketch: S,
}

/// Multi-sorting algorithm for finding pairs of similar short substrings from large-scale string data
/// https://doi.org/10.1007/s10115-009-0271-6
#[derive(Clone, Debug)]
pub struct MultiSort<S> {
    radius: usize,
    num_blocks: usize,
    masks: Vec<S>,
    offsets: Vec<usize>,
    // Buffers for radix sort
    buckets: RefCell<[usize; SORT_MASK + 1]>,
    sorted: RefCell<Vec<Record<S>>>,
}

impl<S> MultiSort<S>
where
    S: Sketch,
{
    /// Reports all similar pairs whose Hamming distance is within `radius`.
    pub fn similar_pairs(sketches: &[S], radius: usize, num_blocks: usize) -> Vec<(usize, usize)> {
        assert!(radius <= num_blocks);
        assert!(num_blocks <= S::dim());

        let (masks, offsets) = Self::build_masks_and_offsets(num_blocks);
        let buckets = [0usize; SORT_MASK + 1];
        let sorted = Vec::with_capacity(sketches.len());

        let this = Self {
            radius,
            num_blocks,
            masks,
            offsets,
            buckets: RefCell::new(buckets),
            sorted: RefCell::new(sorted),
        };

        let mut records: Vec<_> = sketches
            .iter()
            .enumerate()
            .map(|(id, &sketch)| Record { id, sketch })
            .collect();
        let mut results = vec![];
        this.similar_pairs_recur(&mut records, Bitset64::new(), &mut results);
        results
    }

    fn build_masks_and_offsets(num_blocks: usize) -> (Vec<S>, Vec<usize>) {
        let mut masks = vec![S::default(); num_blocks];
        let mut offsets = vec![0; num_blocks + 1];
        let mut i = 0;
        for (b, mask) in masks.iter_mut().enumerate().take(num_blocks) {
            let dim = (b + S::dim()) / num_blocks;
            *mask = S::mask(i..i + dim);
            i += dim;
            offsets[b + 1] = i;
        }
        (masks, offsets)
    }

    fn similar_pairs_recur(
        &self,
        records: &mut [Record<S>],
        blocks: Bitset64,
        results: &mut Vec<(usize, usize)>,
    ) {
        if blocks.len() == self.num_blocks - self.radius {
            self.verify_all_pairs(records, blocks, results);
            return;
        }

        let mut ranges = vec![];
        let max_block = blocks.max().map(|x| x + 1).unwrap_or(0);

        for b in max_block..self.num_blocks {
            self.sort_sketches(b, records);
            self.collision_ranges(b, records, &mut ranges);
            for r in ranges.iter().cloned() {
                self.similar_pairs_recur(&mut records[r], blocks.add(b), results);
            }
        }
    }

    fn verify_all_pairs(
        &self,
        records: &[Record<S>],
        blocks: Bitset64,
        results: &mut Vec<(usize, usize)>,
    ) {
        for i in 0..records.len() {
            let x = &records[i];
            for y in records.iter().skip(i + 1) {
                debug_assert!(self.debug_block_collisions(x.sketch, y.sketch, blocks));
                if x.sketch.hamdist(y.sketch) <= self.radius
                    && self.check_canonical(x.sketch, y.sketch, blocks)
                {
                    debug_assert_ne!(x.id, y.id);
                    // Keeps the order to ease debug.
                    results.push((x.id.min(y.id), x.id.max(y.id)));
                }
            }
        }
    }

    fn check_canonical(&self, x: S, y: S, blocks: Bitset64) -> bool {
        let max = blocks.max().unwrap_or(0);
        let others = blocks.inverse();
        for b in others.iter() {
            if max <= b {
                break;
            }
            if x & self.masks[b] == y & self.masks[b] {
                return false;
            }
        }
        true
    }

    fn sort_sketches(&self, block_id: usize, records: &mut [Record<S>]) {
        if records.len() < 1_000 {
            self.quick_sort_sketches(block_id, records);
        } else {
            self.radix_sort_sketches(block_id, records);
        }
    }

    fn quick_sort_sketches(&self, block_id: usize, records: &mut [Record<S>]) {
        let mask = self.masks[block_id];
        records.sort_unstable_by(|x, y| (x.sketch & mask).cmp(&(y.sketch & mask)));
    }

    fn radix_sort_sketches(&self, block_id: usize, records: &mut [Record<S>]) {
        let mut buckets = self.buckets.borrow_mut();
        let mut sorted = self.sorted.borrow_mut();
        sorted.resize(records.len(), Record::<S>::default());

        let mask = self.masks[block_id];
        for j in (self.offsets[block_id]..self.offsets[block_id + 1]).step_by(SORT_SHIFT) {
            buckets.fill(0);
            for x in records.iter() {
                let k = ((x.sketch & mask) >> j).to_usize().unwrap() & SORT_MASK;
                buckets[k] += 1;
            }
            for k in 1..buckets.len() {
                buckets[k] += buckets[k - 1];
            }
            for x in records.iter().rev() {
                let k = ((x.sketch & mask) >> j).to_usize().unwrap() & SORT_MASK;
                buckets[k] -= 1;
                sorted[buckets[k]] = x.clone();
            }
            for i in 0..records.len() {
                records[i] = sorted[i].clone();
            }
        }
    }

    fn collision_ranges(
        &self,
        block_id: usize,
        records: &[Record<S>],
        ranges: &mut Vec<Range<usize>>,
    ) {
        ranges.clear();
        let mut i = 0;
        for j in 1..records.len() {
            let mask = self.masks[block_id];
            let x = records[i].sketch & mask;
            let y = records[j].sketch & mask;
            if x == y {
                continue;
            }
            if 2 <= j - i {
                ranges.push(i..j);
            }
            i = j;
        }
        let j = records.len();
        if 2 <= j - i {
            ranges.push(i..j);
        }
    }

    fn debug_block_collisions(&self, x: S, y: S, blocks: Bitset64) -> bool {
        for b in blocks.iter() {
            let mx = x & self.masks[b];
            let my = y & self.masks[b];
            if mx != my {
                return false;
            }
        }
        true
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

    fn naive_search(sketches: &[u16], radius: usize) -> Vec<(usize, usize)> {
        let mut results = vec![];
        for i in 0..sketches.len() {
            let x = sketches[i];
            for j in i + 1..sketches.len() {
                let y = sketches[j];
                if x.hamdist(y) <= radius {
                    results.push((i, j));
                }
            }
        }
        results
    }

    fn test_similar_pairs(radius: usize, num_blocks: usize) {
        let sketches = example_sketches();
        let expected = naive_search(&sketches, radius);
        let mut results = MultiSort::similar_pairs(&sketches, radius, num_blocks);
        results.sort_unstable();
        assert_eq!(results, expected);
    }

    #[test]
    fn test_similar_pairs_for_all() {
        for radius in 0..=16 {
            for num_blocks in radius..=16 {
                test_similar_pairs(radius, num_blocks);
            }
        }
    }
}
