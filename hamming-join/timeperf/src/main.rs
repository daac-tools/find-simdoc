use std::time::Instant;

use hamming_join::chunked_join::ChunkedJoiner;
use hamming_join::simple_join::SimpleJoiner;

const TRIALS: usize = 1;
const MIN_SKETCHES: usize = 1_000;
const MAX_SKETCHES: usize = 1_000_000;
const MIN_CHUNKS: usize = 1;
const MAX_CHUNKS: usize = 4;
const RADII: [f64; 4] = [0.01, 0.02, 0.05, 0.1];

macro_rules! timeperf_common {
    ($percent:expr, $name:expr, $method:ident, $sketches:ident, $radii:ident) => {
        let mut num_chunks = MIN_CHUNKS;
        while num_chunks <= MAX_CHUNKS {
            let mut joiner = $method::new(num_chunks);
            let mut num_sketches = MIN_SKETCHES;
            while num_sketches <= MAX_SKETCHES {
                while joiner.num_sketches() < num_sketches {
                    let sketch = &$sketches[joiner.num_sketches()];
                    joiner.add(sketch.iter().cloned());
                }
                for &radius in $radii {
                    let mut num_results = 0;
                    let elapsed_sec = measure(TRIALS, || {
                        num_results += joiner.similar_pairs(radius).len();
                    });
                    num_results /= TRIALS;
                    println!(
                        "[percent={},method={},num_chunks={num_chunks},num_sketches={num_sketches},radius={radius},num_results={num_results}] {elapsed_sec} sec",
                        $percent, $name
                    );
                }
                num_sketches *= 10;
            }
            num_chunks *= 2;
        }
    };
}

fn main() {
    main_percent(50);
    main_percent(70);
}

fn main_percent(percent: u64) {
    let mut sketches = Vec::with_capacity(MAX_SKETCHES);
    for _ in 0..MAX_SKETCHES {
        let mut chunks = Vec::with_capacity(MAX_CHUNKS);
        for _ in 0..MAX_CHUNKS {
            chunks.push((0..64).fold(0u64, |acc, _| {
                let x = rand::random::<u64>() & 100;
                (acc << 1) | ((x < percent) as u64)
            }));
        }
        sketches.push(chunks);
    }
    {
        let radii = &RADII[..1];
        timeperf_common!(percent, "simple_join", SimpleJoiner, sketches, radii);
    }
    {
        let radii = &RADII[..];
        timeperf_common!(percent, "chunked_join", ChunkedJoiner, sketches, radii);
    }
}

fn measure<F>(num_trials: usize, mut func: F) -> f64
where
    F: FnMut(),
{
    // Measure
    let start = Instant::now();
    for _ in 0..num_trials {
        func();
    }
    let duration = start.elapsed();
    duration.as_secs_f64() / num_trials as f64
}