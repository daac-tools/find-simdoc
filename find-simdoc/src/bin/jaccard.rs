use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;
use find_simdoc::feature::{FeatureConfig, FeatureExtractor};
use hamming_join::simple_join::SimpleJoiner;
use lsh::minhash::MinHasher;

#[derive(Parser, Debug)]
#[clap(
    name = "find-simdoc-jaccard",
    about = "A program to find similar documents in the Jaccard space."
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
}

fn main() {
    let args = Args::parse();

    let text_path = args.text_path;
    let radius = args.radius;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let num_chunks = args.num_chunks;

    let texts = BufReader::new(File::open(text_path).unwrap())
        .lines()
        .map(|line| line.unwrap());

    let config = FeatureConfig::new(window_size, delimiter, 53);
    let results = find_in_jaccard(texts, radius, num_chunks, config);

    println!("i,j,dist");
    for (i, j, dist) in results {
        println!("{i},{j},{dist}");
    }
}

fn find_in_jaccard<I, S>(
    texts: I,
    radius: f64,
    num_chunks: usize,
    config: FeatureConfig,
) -> Vec<(usize, usize, f64)>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let hasher = MinHasher::new(42);
    let mut extractor = FeatureExtractor::new(config);
    let mut joiner = SimpleJoiner::<u64>::new(num_chunks);

    let mut features = vec![];
    for text in texts {
        extractor.extract(text.as_ref(), &mut features);
        joiner.add(hasher.iter(&features));
    }

    // In 1-bit minhash, the collision probability is multiplied by 2 over the original.
    // Thus, we should search with the half of the actual radius.
    let mut results = joiner.similar_pairs(radius / 2.);

    // Modifies the distances.
    results.iter_mut().for_each(|(_, _, d)| *d *= 2.);
    results
}
