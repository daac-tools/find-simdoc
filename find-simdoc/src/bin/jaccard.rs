use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

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

    assert_ne!(window_size, 0);

    let texts = BufReader::new(File::open(document_path)?)
        .lines()
        .map(|line| line.unwrap());

    let mut seeder =
        rand_xoshiro::SplitMix64::seed_from_u64(seed.unwrap_or_else(rand::random::<u64>));

    let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64());
    let results = find_in_jaccard(texts, radius, num_chunks, seeder.next_u64(), config)?;

    println!("i,j,dist");
    for (i, j, dist) in results {
        println!("{i},{j},{dist}");
    }

    Ok(())
}

fn find_in_jaccard<I, S>(
    texts: I,
    radius: f64,
    num_chunks: usize,
    seed: u64,
    config: FeatureConfig,
) -> Result<Vec<(usize, usize, f64)>, Box<dyn Error>>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let hasher = MinHasher::new(seed);
    let mut extractor = FeatureExtractor::new(config);
    let mut joiner = ChunkedJoiner::<u64>::new(num_chunks).shows_progress(true);

    eprintln!("[find_in_jaccard] Converting texts into sketches...");
    let mut features = vec![];
    for text in texts {
        let text = text.as_ref();
        assert!(!text.is_empty());
        extractor.extract(text, &mut features);
        joiner.add(hasher.iter(&features))?;
    }
    let memory_in_bytes = joiner.memory_in_bytes() as f64;
    eprintln!(
        "[find_in_jaccard] Produced {} sketches in {} MiB",
        joiner.num_sketches(),
        memory_in_bytes / (1024. * 1024.)
    );

    // In 1-bit minhash, the collision probability is multiplied by 2 over the original.
    // Thus, we should search with the half of the actual radius.
    let mut results = joiner.similar_pairs(radius / 2.);

    // Modifies the distances.
    results.iter_mut().for_each(|(_, _, d)| *d *= 2.);
    Ok(results)
}
