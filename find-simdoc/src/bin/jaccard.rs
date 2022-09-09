use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use find_simdoc::feature::{FeatureConfig, FeatureExtractor};
use hamming_join::chunked_join::ChunkedJoiner;
use lsh::minhash::MinHasher;
use rand::{RngCore, SeedableRng};

#[derive(Parser, Debug)]
#[clap(
    name = "find-simdoc-jaccard",
    about = "A program to find similar documents in the Jaccard space."
)]
struct Args {
    /// File path to a document file to be searched.
    #[clap(short = 'i', long)]
    document_path: PathBuf,

    /// Search radius in the range of [0,1].
    #[clap(short = 'r', long)]
    radius: f64,

    /// Delimiter for recognizing words as tokens in feature extraction.
    /// If None, characters are used for tokens.
    #[clap(short = 'd', long)]
    delimiter: Option<char>,

    /// Window size for w-shingling in feature extraction (must to be more than 0).
    #[clap(short = 'w', long, default_value = "1")]
    window_size: usize,

    /// Number of chunks in sketches, indicating that the number of dimensions in the Hamming space
    /// will be 64*#chunks. The larger this value, the more accurate the approximation,
    /// but the more time and memory it takes to search.
    #[clap(short = 'c', long, default_value = "8")]
    num_chunks: usize,

    /// Seed value for random values.
    #[clap(short = 's', long)]
    seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let document_path = args.document_path;
    let radius = args.radius;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let num_chunks = args.num_chunks;
    let seed = args.seed;

    if window_size == 0 {
        return Err("window_size must not be 0.".into());
    }

    let texts = BufReader::new(File::open(document_path)?)
        .lines()
        .map(|line| line.unwrap());

    let mut seeder =
        rand_xoshiro::SplitMix64::seed_from_u64(seed.unwrap_or_else(rand::random::<u64>));
    let mut joiner = ChunkedJoiner::<u64>::new(num_chunks).shows_progress(true);

    // TODO: Multi-threading.
    {
        let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64());
        let hasher = MinHasher::new(seeder.next_u64());
        let mut extractor = FeatureExtractor::new(config);

        eprintln!("Converting sentences into sketches...");
        let start = Instant::now();
        let mut feature = vec![];
        for (i, text) in texts.enumerate() {
            if (i + 1) % 1000 == 0 {
                eprintln!("Processed {} sentences...", i + 1);
            }
            assert!(!text.is_empty());
            extractor.extract(text, &mut feature);
            joiner.add(hasher.iter(&feature))?;
        }
        let duration = start.elapsed();
        let memory_in_bytes = joiner.memory_in_bytes() as f64;
        eprintln!(
            "Produced {} sketches in {} sec, consuming {} MiB",
            joiner.num_sketches(),
            duration.as_secs_f64(),
            memory_in_bytes / (1024. * 1024.)
        );
    }

    eprintln!("Finding all similar pairs in sketches...");
    let start = Instant::now();
    // In 1-bit minhash, the collision probability is multiplied by 2 over the original.
    // Thus, we should search with the half of the actual radius.
    let mut results = joiner.similar_pairs(radius / 2.);
    // Modifies the distances.
    results.iter_mut().for_each(|(_, _, d)| *d *= 2.);
    eprintln!("Done in {} sec", start.elapsed().as_secs_f64());

    println!("i,j,dist");
    for (i, j, dist) in results {
        println!("{i},{j},{dist}");
    }

    Ok(())
}
