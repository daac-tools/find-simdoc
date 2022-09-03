pub mod feature;
pub mod shingling;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::Parser;
use feature::{FeatureConfig, FeatureExtractor};
use hamming_join::simple_join::SimpleJoiner;
use lsh::minhash::MinHasher;

#[derive(Clone, Debug)]
enum Metric {
    Jaccard,
    Cosine,
}

impl FromStr for Metric {
    type Err = &'static str;
    fn from_str(metric: &str) -> Result<Self, Self::Err> {
        match metric {
            "jaccard" => Ok(Metric::Jaccard),
            "cosine" => Ok(Metric::Cosine),
            _ => Err("Could not parse a metric option"),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "find-simdoc", about = "A program to find similar documents.")]
struct Args {
    #[clap(short = 'i', long, action)]
    text_path: PathBuf,

    #[clap(short = 'm', long, action)]
    metric: Metric,

    #[clap(short = 'r', long, action)]
    radius: f64,

    #[clap(short = 'd', long, action)]
    delimiter: Option<char>,

    #[clap(short = 'w', long, action)]
    window_size: usize,
}

fn main() {
    let args = Args::parse();

    let text_path = args.text_path;
    let _metric = args.metric;
    let radius = args.radius;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let num_chunks = 64;

    let texts = load_lines(text_path);
    println!("#texts = {}", texts.len());

    let mut extractor = FeatureExtractor::new(FeatureConfig::new(window_size, delimiter, 53));
    let mut joiner = SimpleJoiner::<u64>::new(num_chunks);
    let hasher = MinHasher::new(42);

    for text in &texts {
        let features = extractor.extract(text);
        joiner.add(hasher.iter(features));
    }

    let results = joiner.similar_pairs(radius);
    for (i, j, d) in results {
        println!("[i={i},j={j},dist={d}]");
        println!("{}", texts[i]);
        println!("{}", texts[j]);
    }
}

fn load_lines<P>(path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    buf.lines().map(|line| line.unwrap()).collect()
}
