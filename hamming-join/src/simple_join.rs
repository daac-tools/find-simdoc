use crate::sketch::Sketch;

pub struct SimpleJoiner<S> {
    sketches: Vec<Vec<S>>,
    num_chunks: usize,
}

impl<S> SimpleJoiner<S>
where
    S: Sketch,
{
    pub fn new(num_chunks: usize) -> Self {
        Self {
            sketches: vec![],
            num_chunks,
        }
    }

    pub fn add<I>(&mut self, sketch: I)
    where
        I: IntoIterator<Item = S>,
    {
        let mut iter = sketch.into_iter();
        let mut sketch = Vec::with_capacity(self.num_chunks());
        for _ in 0..self.num_chunks() {
            sketch.push(iter.next().unwrap())
        }
        self.sketches.push(sketch);
    }

    pub fn similar_pairs(&self, radius: f64) -> Vec<(usize, usize, f64)> {
        let dimension = S::dim() * self.num_chunks();
        let hamdist = (dimension as f64 * radius).ceil() as usize;
        println!("dimension={dimension}, hamdist={hamdist}");

        let mut matched = vec![];
        for i in 0..self.sketches.len() {
            for j in i + 1..self.sketches.len() {
                let dist = self.hamming_distance(i, j);
                let dist = dist as f64 / dimension as f64;
                if dist <= radius {
                    matched.push((i, j, dist));
                }
            }
        }
        matched
    }

    pub fn num_chunks(&self) -> usize {
        self.num_chunks
    }

    fn hamming_distance(&self, i: usize, j: usize) -> usize {
        let xs = &self.sketches[i];
        let ys = &self.sketches[j];
        let mut dist = 0;
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            dist += x.hamdist(y);
        }
        dist
    }
}
