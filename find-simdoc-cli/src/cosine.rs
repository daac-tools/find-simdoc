use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use find_simdoc::tfidf::{Idf, Tf};
use find_simdoc::CosineSearcher;

use clap::Parser;

#[derive(Clone, Debug, PartialEq, Eq)]
enum TfWeights {
    Binary,
    Standard,
    Sublinear,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Empty lines must not be included.
    #[clap(short = 'i', long)]
    document_path: PathBuf,

    /// Search radius in the range of [0,1].
    #[clap(short = 'r', long)]
    radius: f64,

    /// Delimiter for recognizing words as tokens in feature extraction.
    /// If None, characters are used for tokens.
    #[clap(short = 'd', long)]
    delimiter: Option<char>,

    /// Window size for w-shingling in feature extraction (must be more than 0).
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

    /// Disables parallel construction.
    #[clap(short = 'p', long)]
    disable_parallel: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let document_path = args.document_path;
    let radius = args.radius;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let num_chunks = args.num_chunks;
    let tf_weight = args.tf;
    let idf_weight = args.idf;
    let seed = args.seed;
    let disable_parallel = args.disable_parallel;

    let mut searcher = CosineSearcher::new(window_size, delimiter, seed)?.shows_progress(true);

    let tf = match tf_weight {
        TfWeights::Binary => None,
        TfWeights::Standard | TfWeights::Sublinear => {
            Some(Tf::new().sublinear(tf_weight == TfWeights::Sublinear))
        }
    };

    let idf = match idf_weight {
        IdfWeights::Unary => None,
        IdfWeights::Standard | IdfWeights::Smooth => {
            eprintln!("Building IDF...");
            let start = Instant::now();
            let documents = texts_iter(File::open(&document_path)?);
            let idf = Idf::new()
                .smooth(idf_weight == IdfWeights::Smooth)
                .build(documents, searcher.config())?;
            let duration = start.elapsed();
            eprintln!("Produced in {} sec", duration.as_secs_f64());
            Some(idf)
        }
    };

    searcher = searcher.tf(tf).idf(idf);

    {
        eprintln!("Converting documents into sketches...");
        let start = Instant::now();
        let documents = texts_iter(File::open(&document_path)?);
        searcher = if disable_parallel {
            searcher.build_sketches(documents, num_chunks)?
        } else {
            searcher.build_sketches_in_parallel(documents, num_chunks)?
        };
        let duration = start.elapsed();
        let memory_in_bytes = searcher.memory_in_bytes() as f64;
        eprintln!(
            "Produced {} sketches in {} sec, consuming {} MiB",
            searcher.len(),
            duration.as_secs_f64(),
            memory_in_bytes / (1024. * 1024.)
        );
    }

    eprintln!("Finding all similar pairs in sketches...");
    let start = Instant::now();
    let results = searcher.search_similar_pairs(radius);
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
