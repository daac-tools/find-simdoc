use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;
use find_simdoc::feature::{FeatureConfig, FeatureExtractor};
use hamming_join::sketch::Sketch;
use lsh::minhash::MinHasher;
use rand::{RngCore, SeedableRng};

#[derive(Parser, Debug)]
#[clap(
    name = "find-simdoc-minhash_mae",
    about = "A program to test mean absolute errors in 1-bit minwise hashing."
)]
struct Args {
    /// File path to a document file to be searched.
    #[clap(short = 'i', long)]
    document_path: PathBuf,

    /// Delimiter for recognizing words as tokens in feature extraction.
    /// If None, characters are used for tokens.
    #[clap(short = 'd', long)]
    delimiter: Option<char>,

    /// Window size for w-shingling in feature extraction (must to be more than 0).
    #[clap(short = 'w', long, default_value = "1")]
    window_size: usize,

    /// Seed value for random values.
    #[clap(short = 's', long)]
    seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let document_path = args.document_path;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let seed = args.seed;

    assert_ne!(window_size, 0);

    let texts = BufReader::new(File::open(document_path)?)
        .lines()
        .map(|line| line.unwrap());

    let mut seeder =
        rand_xoshiro::SplitMix64::seed_from_u64(seed.unwrap_or_else(rand::random::<u64>));

    let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64());
    let mut extractor = FeatureExtractor::new(config);

    eprintln!("Loading texts and extracting features...");
    let mut features = vec![];
    for text in texts {
        assert!(!text.is_empty());
        let mut feature = vec![];
        extractor.extract(text, &mut feature);
        features.push(feature);
    }
    let n = features.len();
    eprintln!("Extracted {n} features ({} pairs)", n * (n - 1) / 2);

    eprintln!("Producing binary sketches...");
    let hasher = MinHasher::new(seeder.next_u64());
    let mut sketches = vec![];
    for (i, feature) in features.iter().enumerate() {
        if (i + 1) % 100 == 0 {
            eprintln!("Processed {}/{}...", i + 1, n);
        }
        let mut iter = hasher.iter(feature);
        let mut sketch = Vec::with_capacity(100);
        (0..100).for_each(|_| sketch.push(iter.next().unwrap()));
        sketches.push(sketch);
    }

    eprintln!("Computing exact Jaccard distances...");
    let mut jac_dists = vec![];
    for i in 0..n {
        if (i + 1) % 100 == 0 {
            eprintln!("Processed {}/{}...", i + 1, n);
        }
        let x = &features[i];
        for j in i + 1..n {
            let y = &features[j];
            jac_dists.push(lsh::jaccard_distance(x.iter().clone(), y.iter().clone()));
        }
    }

    eprintln!("Computing Hamming distances...");
    println!("num_chunks,dimensions,mean_absolute_error");
    for num_chunks in 1..=100 {
        let mut sum_error = 0.;
        let mut jac_dist_iter = jac_dists.iter();
        for i in 0..n {
            let x = &sketches[i];
            for j in i + 1..n {
                let y = &sketches[j];
                let jac_dist = *jac_dist_iter.next().unwrap();
                let ham_dist = hamming_distance(&x[..num_chunks], &y[..num_chunks]);
                sum_error += (jac_dist - ham_dist).abs();
            }
        }
        assert_eq!(jac_dist_iter.next(), None);
        let dim = num_chunks * 64;
        let mae = sum_error / jac_dists.len() as f64;
        println!("{num_chunks},{dim},{mae}");
    }

    Ok(())
}

fn hamming_distance(xs: &[u64], ys: &[u64]) -> f64 {
    assert_eq!(xs.len(), ys.len());
    let mut dist = 0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        dist += x.hamdist(y);
    }
    // In 1-bit minhash, the collision probability is multiplied by 2 over the original.
    // Thus, we should modify the Hamming distance with a factor of 2.
    dist as f64 / (xs.len() * 64) as f64 * 2.
}
