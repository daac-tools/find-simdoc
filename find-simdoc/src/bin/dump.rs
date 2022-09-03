use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "find-simdoc-dump", about = "A program to dump similar texts.")]
struct Args {
    #[clap(short = 'i', long)]
    text_path: PathBuf,

    #[clap(short = 's', long)]
    simpair_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let text_path = args.text_path;
    let simpair_path = args.simpair_path;

    let texts: Vec<_> = BufReader::new(File::open(text_path)?)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    for (i, row) in BufReader::new(File::open(simpair_path)?)
        .lines()
        .enumerate()
    {
        if i == 0 {
            continue;
        }
        let row = row?;
        let cols: Vec<_> = row.split(',').collect();
        let i = cols[0].parse::<usize>()?;
        let j = cols[1].parse::<usize>()?;
        let dist = cols[2].parse::<f64>()?;
        println!("[i={i},j={j},dist={dist}]");
        println!("{}", texts[i]);
        println!("{}", texts[j]);
    }

    Ok(())
}
