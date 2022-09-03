use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use clap::Parser;
use find_simdoc::feature::{FeatureConfig, FeatureExtractor};
use hamming_join::simple_join::SimpleJoiner;
use lsh::simhash::SimHasher;

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

    #[clap(short = 'c', long, default_value = "64")]
    num_chunks: usize,
}

fn main() {
    let args = Args::parse();

    let text_path = args.text_path;
    let radius = args.radius;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let num_chunks = args.num_chunks;

    let texts = load_lines(text_path);
    println!("#texts = {}", texts.len());

    let config = FeatureConfig::new(window_size, delimiter, 53);
    let results = find_in_cosine(texts.iter().clone(), radius, num_chunks, config);
    println!("#results = {}", results.len());

    for (i, j, d) in results {
        println!("[i={i},j={j},dist={d}]");
        println!("{}", texts[i]);
        println!("{}", texts[j]);
    }
}

fn find_in_cosine<I, S>(
    texts: I,
    radius: f64,
    num_chunks: usize,
    config: FeatureConfig,
) -> Vec<(usize, usize, f64)>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let hasher = SimHasher::new(42);
    let mut extractor = FeatureExtractor::new(config);
    let mut joiner = SimpleJoiner::<u64>::new(num_chunks);

    let mut features = vec![];
    for text in texts {
        extractor.extract_with_weights(text.as_ref(), &mut features);
        joiner.add(hasher.iter(&features));
    }
    joiner.similar_pairs(radius)
}

fn load_lines<P>(path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    buf.lines().map(|line| line.unwrap()).collect()
}
