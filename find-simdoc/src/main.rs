pub mod shingling;

use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

use shingling::ShingleIter;

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
    #[clap(short = 'm', long)]
    metric: Metric,

    #[clap(short = 'r', long)]
    radius: f64,

    #[clap(short = 'd', long)]
    delimiter: Option<String>,

    #[clap(short = 'q', long)]
    qgram: usize,
}

fn main() {
    let args = Args::parse();
    let metric = args.metric;
    let radius = args.radius;
    let delimiter = args.delimiter;
    let qgram = args.qgram;

    let mut texts = vec![];
    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        texts.push(line);
    }
    println!("#texts = {}", texts.len());

    for text in &texts {}
}

fn tokenize() {}
