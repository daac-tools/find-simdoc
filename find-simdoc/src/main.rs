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
            "jac" => Ok(Self::Jaccard),
            "cos" => Ok(Self::Cosine),
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

    #[clap(short = 'c', long, action, default_value = "64")]
    num_chunks: usize,
}

fn main() {
    let args = Args::parse();

    let text_path = args.text_path;
    let _metric = args.metric;
    let radius = args.radius;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let num_chunks = args.num_chunks;

    let texts = load_lines(text_path);
    println!("#texts = {}", texts.len());

    let feature_config = FeatureConfig::new(window_size, delimiter, 53);
    let results = find_in_jaccard(texts.iter().clone(), radius, num_chunks, feature_config);

    let mut extractor = FeatureExtractor::new(feature_config);
    for (i, j, d) in results {
        let ti = &texts[i];
        let tj = &texts[j];
        let fi = extractor.extract(ti).to_vec();
        let fj = extractor.extract(tj).to_vec();
        let actual = lsh::jaccard_distance(fi, fj);
        println!("[i={i},j={j},dist={d},act={actual}]");
        println!("{}", texts[i]);
        println!("{}", texts[j]);
    }
}

fn find_in_jaccard<I, S>(
    texts: I,
    radius: f64,
    num_chunks: usize,
    feature_config: FeatureConfig,
) -> Vec<(usize, usize, f64)>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let mut extractor = FeatureExtractor::new(feature_config);
    let mut joiner = SimpleJoiner::<u64>::new(num_chunks);

    let hasher = MinHasher::new(42);
    for text in texts {
        let features = extractor.extract(text.as_ref());
        joiner.add(hasher.iter(features));
    }

    // In 1-bit minhash, the collision probability is multiplied by 2 over the original.
    // Thus, we should search with the half of the actual radius.
    let mut results = joiner.similar_pairs(radius / 2.);

    // Modifies the distances.
    results.iter_mut().for_each(|(_, _, d)| *d *= 2.);
    results
}

fn load_lines<P>(path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    buf.lines().map(|line| line.unwrap()).collect()
}
