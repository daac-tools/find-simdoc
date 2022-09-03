use std::time::Duration;

use rand::{thread_rng, Rng};

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion, SamplingMode,
};

use hamming_join::simple_join::SimpleJoiner;

const SAMPLE_SIZE: usize = 10;
const WARM_UP_TIME: Duration = Duration::from_secs(5);
const MEASURE_TIME: Duration = Duration::from_secs(10);

const MIN_SKETCHES: usize = 1000;
const MAX_SKETCHES: usize = 100000;
const MIN_CHUNKS: usize = 1;
const MAX_CHUNKS: usize = 4;
const RADII: [f64; 4] = [0.01, 0.02, 0.05, 0.1];

fn criterion_uniform(c: &mut Criterion) {
    let mut group = c.benchmark_group("uniform");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(WARM_UP_TIME);
    group.measurement_time(MEASURE_TIME);
    group.sampling_mode(SamplingMode::Flat);

    let mut rng = thread_rng();
    let mut sketches = Vec::with_capacity(MAX_SKETCHES);
    for _ in 0..MAX_SKETCHES {
        let mut chunks = Vec::with_capacity(MAX_CHUNKS);
        for _ in 0..MAX_CHUNKS {
            chunks.push(rng.gen::<u64>());
        }
        sketches.push(chunks);
    }

    add_join_benches(&mut group, sketches);
}

macro_rules! bench_common {
    ($name:expr, $method:ident, $sketches:ident, $group:ident) => {
        let mut num_chunks = MIN_CHUNKS;
        while num_chunks <= MAX_CHUNKS {
            let mut joiner = $method::new(num_chunks);
            let mut num_sketches = MIN_SKETCHES;
            while num_sketches <= MAX_SKETCHES {
                while joiner.num_sketches() < num_sketches {
                    let sketch = &$sketches[joiner.num_sketches()];
                    joiner.add(sketch.iter().cloned());
                }
                for &radius in &RADII {
                    let bench_name = format!("{}/{num_chunks}/{num_sketches}/{radius}", $name);
                    $group.bench_function(bench_name, |b| {
                        b.iter(|| {
                            if joiner.similar_pairs(0.05).len() == usize::MAX {
                                panic!();
                            }
                        });
                    });
                }
                num_sketches *= 10;
            }
            num_chunks *= 2;
        }
    };
}

fn add_join_benches(group: &mut BenchmarkGroup<WallTime>, sketches: Vec<Vec<u64>>) {
    bench_common!("simple_join", SimpleJoiner, sketches, group);
}

criterion_group!(benches, criterion_uniform);
criterion_main!(benches);
