use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;

use find_simdoc::jaccard::JaccardSearcher;

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

    let mut searcher = JaccardSearcher::new(window_size, delimiter, seed).shows_progress(true);

    {
        eprintln!("Converting documents into sketches...");
        let start = Instant::now();
        let documents = texts_iter(File::open(&document_path)?);
        searcher = searcher.build_sketches(documents, num_chunks);
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
