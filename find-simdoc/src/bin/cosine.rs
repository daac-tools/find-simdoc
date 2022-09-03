use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;
use find_simdoc::feature::{FeatureConfig, FeatureExtractor};
use hamming_join::simple_join::SimpleJoiner;
use lsh::simhash::SimHasher;
use rand::{RngCore, SeedableRng};

#[derive(Parser, Debug)]
#[clap(
    name = "find-simdoc-cosine",
    about = "A program to find similar documents in the Cosine space."
)]
struct Args {
    #[clap(short = 'i', long)]
    text_path: PathBuf,

    #[clap(short = 'r', long)]
    radius: f64,

    #[clap(short = 'd', long)]
    delimiter: Option<char>,

    #[clap(short = 'w', long)]
    window_size: usize,

    #[clap(short = 'c', long, default_value = "8")]
    num_chunks: usize,

    #[clap(short = 's', long)]
    seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let text_path = args.text_path;
    let radius = args.radius;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let num_chunks = args.num_chunks;
    let seed = args.seed;

    let texts = BufReader::new(File::open(text_path)?)
        .lines()
        .map(|line| line.unwrap());

    let mut seeder = rand_xoshiro::SplitMix64::seed_from_u64(seed.unwrap_or(rand::random::<u64>()));

    let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64());
    let results = find_in_cosine(texts, radius, num_chunks, seeder.next_u64(), config);

    println!("i,j,dist");
    for (i, j, dist) in results {
        println!("{i},{j},{dist}");
    }

    Ok(())
}

fn find_in_cosine<I, S>(
    texts: I,
    radius: f64,
    num_chunks: usize,
    seed: u64,
    config: FeatureConfig,
) -> Vec<(usize, usize, f64)>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let hasher = SimHasher::new(seed);
    let mut extractor = FeatureExtractor::new(config);
    let mut joiner = SimpleJoiner::<u64>::new(num_chunks);

    let mut features = vec![];
    for text in texts {
        extractor.extract_with_weights(text.as_ref(), &mut features);
        joiner.add(hasher.iter(&features));
    }
    joiner.similar_pairs(radius)
}