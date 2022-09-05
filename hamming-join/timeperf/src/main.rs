use std::time::Instant;

use hamming_join::chunked_join::ChunkedJoiner;
use hamming_join::simple_join::SimpleJoiner;

const TRIALS: usize = 1;
const SCALES: [usize; 4] = [1_000, 10_000, 100_000, 1_000_000];
const CHUNKS: [usize; 3] = [1, 2, 4];
const RADII: [f64; 4] = [0.01, 0.02, 0.05, 0.1];

macro_rules! timeperf_common {
    ($percent:expr, $name:expr, $method:ident, $sketches:ident, $radii:ident, $chunks:ident, $scales:ident) => {
        for &num_chunks in $chunks {
            let mut joiner = $method::new(num_chunks);
            for &num_sketches in $scales {
                while joiner.num_sketches() < num_sketches {
                    let sketch = &$sketches[joiner.num_sketches()];
                    joiner.add(sketch.iter().cloned()).unwrap();
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
            }
        }
    };
}

fn main() {
    main_percent(50);
    main_percent(70);
}

fn main_percent(percent: u64) {
    let max_chunks = *CHUNKS.last().unwrap();
    let max_sketches = *SCALES.last().unwrap();

    let mut sketches = Vec::with_capacity(max_sketches);
    for _ in 0..max_sketches {
        let mut chunks = Vec::with_capacity(max_chunks);
        for _ in 0..max_chunks {
            chunks.push((0..64).fold(0u64, |acc, _| {
                let x = rand::random::<u64>() & 100;
                (acc << 1) | ((x < percent) as u64)
            }));
        }
        sketches.push(chunks);
    }
    {
        let radii = &RADII[..];
        let chunks = &CHUNKS[..];
        let scales = &SCALES[..];
        timeperf_common!(
            percent,
            "chunked_join",
            ChunkedJoiner,
            sketches,
            radii,
            chunks,
            scales
        );
    }
    {
        let radii = &RADII[..1];
        let chunks = &CHUNKS[..];
        let scales = &SCALES[..3];
        timeperf_common!(
            percent,
            "simple_join",
            SimpleJoiner,
            sketches,
            radii,
            chunks,
            scales
        );
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
