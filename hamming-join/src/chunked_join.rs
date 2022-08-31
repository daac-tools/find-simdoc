use crate::multi_sort::MultiSort;
use crate::sketch::Sketch;

pub struct ChunkedJoiner<S> {
    chunks: Vec<Vec<S>>,
}

impl<S> ChunkedJoiner<S>
where
    S: Sketch,
{
    pub fn new(num_chunks: usize) -> Self {
        Self {
            chunks: vec![vec![]; num_chunks],
        }
    }

    pub fn add<I>(&mut self, sketch: I)
    where
        I: IntoIterator<Item = S>,
    {
        let mut iter = sketch.into_iter();
        self.chunks
            .iter_mut()
            .for_each(|chunk| chunk.push(iter.next().unwrap()));
    }

    pub fn similar_pairs(&self, radius: f64) -> Vec<(usize, usize, f64)> {
        let dimension = S::dim() * self.num_chunks();
        let hamdist = (dimension as f64 * radius).ceil() as usize;
        println!("dimension={dimension}, hamdist={hamdist}");

        // Can be threaded.
        let mut candidates = vec![];
        for chunk in &self.chunks {
            let r = hamdist / self.num_chunks();
            let results = MultiSort::similar_pairs(chunk, r, r * 2);
            candidates.extend(results);
        }
        candidates.sort_unstable();
        candidates.dedup();
        println!("#candidates={}", candidates.len());

        let mut matched = vec![];
        for (i, j) in candidates {
            let dist = self.hamming_distance(i, j);
            let dist = dist as f64 / dimension as f64;
            if dist <= radius {
                matched.push((i, j, dist));
            }
        }
        matched
    }

    pub fn num_chunks(&self) -> usize {
        self.chunks.len()
    }

    fn hamming_distance(&self, i: usize, j: usize) -> usize {
        let mut dist = 0;
        for chunk in &self.chunks {
            dist += chunk[i].hamdist(chunk[j]);
        }
        dist
    }
}
