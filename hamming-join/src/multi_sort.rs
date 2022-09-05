use std::cell::RefCell;
use std::ops::Range;

use crate::bitset64::Bitset64;
use crate::sketch::Sketch;

const SORT_SHIFT: usize = 8;
const SORT_MASK: usize = (1 << SORT_SHIFT) - 1;
const DEFAULT_THRESHOLD_IN_SORT: usize = 1000;

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
    // For radix sort
    threshold_in_sort: usize,
    buckets: RefCell<[usize; SORT_MASK + 1]>,
    sorted: RefCell<Vec<Record<S>>>,
}

impl<S> Default for MultiSort<S>
where
    S: Sketch,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> MultiSort<S>
where
    S: Sketch,
{
    pub const fn new() -> Self {
        Self {
            radius: 0,
            num_blocks: 0,
            masks: vec![],
            offsets: vec![],
            threshold_in_sort: DEFAULT_THRESHOLD_IN_SORT,
            buckets: RefCell::new([0usize; SORT_MASK + 1]),
            sorted: RefCell::new(vec![]),
        }
    }

    pub fn num_blocks(mut self, num_blocks: usize) -> Self {
        if num_blocks <= S::dim() {
            self.num_blocks = num_blocks;
        }
        self
    }

    pub fn threshold_in_sort(mut self, threshold_in_sort: usize) -> Self {
        self.threshold_in_sort = threshold_in_sort;
        self
    }

    /// Reports all similar pairs whose Hamming distance is within `radius`.
    pub fn similar_pairs(mut self, sketches: &[S], radius: usize) -> Vec<(usize, usize)> {
        if self.num_blocks == 0 || self.num_blocks < radius {
            // Following Tabei's paper.
            self.num_blocks = S::dim().min(radius + 3);
        }

        self.build_masks_and_offsets();
        self.radius = radius;
        self.sorted = RefCell::new(Vec::with_capacity(sketches.len()));

        let mut records: Vec<_> = sketches
            .iter()
            .enumerate()
            .map(|(id, &sketch)| Record { id, sketch })
            .collect();
        let mut results = vec![];
        self.similar_pairs_recur(&mut records, Bitset64::new(), &mut results);
        results
    }

    fn build_masks_and_offsets(&mut self) {
        let mut masks = vec![S::default(); self.num_blocks];
        let mut offsets = vec![0; self.num_blocks + 1];
        let mut i = 0;
        for (b, mask) in masks.iter_mut().enumerate().take(self.num_blocks) {
            let dim = (b + S::dim()) / self.num_blocks;
            *mask = S::mask(i..i + dim);
            i += dim;
            offsets[b + 1] = i;
        }
        self.masks = masks;
        self.offsets = offsets;
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
        if records.len() < self.threshold_in_sort {
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
        let mut results = MultiSort::new()
            .num_blocks(num_blocks)
            .threshold_in_sort(5)
            .similar_pairs(&sketches, radius);
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
