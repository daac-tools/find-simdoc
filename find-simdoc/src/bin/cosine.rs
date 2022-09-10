use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use all_pairs_hamming::chunked_join::ChunkedJoiner;
use clap::Parser;
use find_simdoc::feature::{FeatureConfig, FeatureExtractor};
use find_simdoc::tfidf::{Idf, Tf};
use lsh::simhash::SimHasher;
use rand::{RngCore, SeedableRng};

#[derive(Clone, Debug)]
enum TfWeights {
    Binary,
    Standard,
    Sublinear,
}

#[derive(Clone, Debug)]
enum IdfWeights {
    Unary,
    Standard,
    Smooth,
}

impl FromStr for TfWeights {
    type Err = &'static str;
    fn from_str(w: &str) -> Result<Self, Self::Err> {
        match w {
            "binary" => Ok(Self::Binary),
            "standard" => Ok(Self::Standard),
            "sublinear" => Ok(Self::Sublinear),
            _ => Err("Could not parse a tf-weighting value"),
        }
    }
}

impl FromStr for IdfWeights {
    type Err = &'static str;
    fn from_str(w: &str) -> Result<Self, Self::Err> {
        match w {
            "unary" => Ok(Self::Unary),
            "standard" => Ok(Self::Standard),
            "smooth" => Ok(Self::Smooth),
            _ => Err("Could not parse a idf-weighting value"),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(
    name = "find-simdoc-cosine",
    about = "A program to find similar documents in the Cosine space."
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

    /// Weighting variant of term frequency.
    /// "binary" is the boolean frequency.
    /// "standard" is the standard term frequency.
    /// "sublinear" is the logarithmically scaled frequency.
    #[clap(short = 'T', long, default_value = "standard")]
    tf: TfWeights,

    /// Weighting variant of inverse document frequency.
    /// "unary" is always 1.
    /// "standard" is the standard inverse document frequency.
    /// "smooth" is the smoothed inverse document frequency.
    #[clap(short = 'I', long, default_value = "smooth")]
    idf: IdfWeights,

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
    let tf_kind = args.tf;
    let idf_kind = args.idf;
    let seed = args.seed;

    if window_size == 0 {
        return Err("window_size must not be 0.".into());
    }

    let mut seeder =
        rand_xoshiro::SplitMix64::seed_from_u64(seed.unwrap_or_else(rand::random::<u64>));
    let mut joiner = ChunkedJoiner::<u64>::new(num_chunks).shows_progress(true);

    {
        let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64());
        let hasher = SimHasher::new(seeder.next_u64());
        let mut extractor = FeatureExtractor::new(config);

        let idf = match idf_kind {
            IdfWeights::Unary => None,
            IdfWeights::Standard | IdfWeights::Smooth => {
                eprintln!("Building IDF...");
                let start = Instant::now();
                let texts = texts_iter(File::open(&document_path)?);
                let mut idf = Idf::new();
                let mut feature = vec![];
                for text in texts {
                    assert!(!text.is_empty());
                    extractor.extract(text, &mut feature);
                    idf.add(&feature);
                }
                let duration = start.elapsed();
                eprintln!(
                    "Produced {} documents in {} sec",
                    idf.num_docs(),
                    duration.as_secs_f64(),
                );
                Some(idf)
            }
        };

        eprintln!("Converting sentences into sketches...");
        let start = Instant::now();
        let texts = texts_iter(File::open(&document_path)?);
        let mut tf = Tf::new();
        let mut feature = vec![];
        for (i, text) in texts.enumerate() {
            if (i + 1) % 1000 == 0 {
                eprintln!("Processed {} sentences...", i + 1);
            }
            assert!(!text.is_empty());
            extractor.extract_with_weights(text, &mut feature);
            match tf_kind {
                TfWeights::Binary => {}
                TfWeights::Standard => {
                    tf.tf(&mut feature);
                }
                TfWeights::Sublinear => {
                    tf.tf_sublinear(&mut feature);
                }
            }
            match idf_kind {
                IdfWeights::Unary => {}
                IdfWeights::Standard => {
                    let idf = idf.as_ref().unwrap();
                    for (term, weight) in feature.iter_mut() {
                        *weight *= idf.idf(*term);
                    }
                }
                IdfWeights::Smooth => {
                    let idf = idf.as_ref().unwrap();
                    for (term, weight) in feature.iter_mut() {
                        *weight *= idf.idf_smooth(*term);
                    }
                }
            }
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
    let results = joiner.similar_pairs(radius);
    eprintln!("Done in {} sec", start.elapsed().as_secs_f64());

    println!("i,j,dist");
    for (i, j, dist) in results {
        println!("{i},{j},{dist}");
    }

    Ok(())
}

fn texts_iter<R>(rdr: R) -> impl Iterator<Item = String>
where
    R: Read,
{
    BufReader::new(rdr).lines().map(|line| line.unwrap())
}
