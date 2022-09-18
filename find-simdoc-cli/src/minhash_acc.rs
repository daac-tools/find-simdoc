use std::convert::TryInto;
use std::error::Error;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use all_pairs_hamming::sketch::Sketch;
use byteorder::{ReadBytesExt, WriteBytesExt};
use clap::Parser;
use find_simdoc::feature::{FeatureConfig, FeatureExtractor};
use find_simdoc::lsh::minhash::MinHasher;
use hashbrown::HashSet;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;

const MAX_CHUNKS: usize = 100;
const TMP_FILENAME: &'static str = "tmp.jac_dist";

#[derive(Parser, Debug)]
#[clap(
    name = "find-simdoc-minhash_acc",
    about = "A program to test accuracy in 1-bit minwise hashing."
)]
struct Args {
    /// File path to a document file to be searched.
    /// Empty lines must not be included.
    #[clap(short = 'i', long)]
    document_path: PathBuf,

    /// Delimiter for recognizing words as tokens in feature extraction.
    /// If None, characters are used for tokens.
    #[clap(short = 'd', long)]
    delimiter: Option<char>,

    /// Window size for w-shingling in feature extraction (must to be more than 0).
    #[clap(short = 'w', long, default_value = "1")]
    window_size: usize,

    /// Seed value for random values.
    #[clap(short = 's', long)]
    seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let document_path = args.document_path;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let seed = args.seed;

    if window_size == 0 {
        return Err("window_size must not be 0.".into());
    }

    let documents = BufReader::new(File::open(document_path)?)
        .lines()
        .map(|line| line.unwrap());

    let mut seeder =
        rand_xoshiro::SplitMix64::seed_from_u64(seed.unwrap_or_else(rand::random::<u64>));

    let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64())?;
    let extractor = FeatureExtractor::new(&config);

    let features = {
        eprintln!("Loading documents and extracting features...");
        let start = Instant::now();
        let mut features = vec![];
        for document in documents {
            if document.is_empty() {
                return Err("Input document must not be empty.".into());
            }
            let mut feature = vec![];
            extractor.extract(document, &mut feature);
            features.push(feature);
        }
        let duration = start.elapsed();
        let total_bytes =
            features.iter().fold(0, |acc, f| acc + f.len()) * std::mem::size_of::<u64>();
        eprintln!(
            "Extracted {} features in {} sec, consuming {} MiB",
            features.len(),
            duration.as_secs_f64(),
            total_bytes as f64 / (1024. * 1024.)
        );
        features
    };

    let sketches = {
        eprintln!("Producing binary sketches...");
        let start = Instant::now();
        let hasher = MinHasher::new(seeder.next_u64());

        #[allow(clippy::mutex_atomic)]
        let processed = Mutex::new(0usize);

        let mut sketches = vec![vec![]; features.len()];
        features
            .par_iter()
            .map(|feature| {
                #[allow(clippy::mutex_atomic)]
                {
                    // Mutex::lock also locks eprintln.
                    let mut cnt = processed.lock().unwrap();
                    *cnt += 1;
                    if *cnt % 1000 == 0 {
                        eprintln!("Processed {} features...", *cnt);
                    }
                }
                let mut iter = hasher.iter(feature);
                let mut sketch = Vec::with_capacity(MAX_CHUNKS);
                (0..MAX_CHUNKS).for_each(|_| sketch.push(iter.next().unwrap()));
                sketch
            })
            .collect_into_vec(&mut sketches);

        let duration = start.elapsed();
        let total_bytes = sketches.len() * MAX_CHUNKS * std::mem::size_of::<u64>();
        eprintln!(
            "Produced in {} sec, consuming {} MiB",
            duration.as_secs_f64(),
            total_bytes as f64 / (1024. * 1024.)
        );
        sketches
    };

    let possible_pairs = {
        let possible_pairs = features.len() * (features.len() - 1) / 2;
        eprintln!("Computing exact Jaccard distances for {possible_pairs} pairs...");
        let start = Instant::now();

        #[allow(clippy::mutex_atomic)]
        {
            let processed = Mutex::new(0usize);
            let writer = Mutex::new(BufWriter::new(File::create(TMP_FILENAME)?));

            (0..features.len()).into_par_iter().for_each(|i| {
                {
                    // Mutex::lock also locks eprintln.
                    let mut cnt = processed.lock().unwrap();
                    *cnt += 1;
                    if *cnt % 100 == 0 {
                        eprintln!("Processed {} features...", *cnt);
                    }
                }
                let mut jac_dists = Vec::with_capacity(features.len() - i);

                let x = &features[i];
                for (j, y) in features.iter().enumerate().skip(i + 1) {
                    let dist =
                        find_simdoc::lsh::jaccard_distance(x.iter().clone(), y.iter().clone());
                    let jac_dist = JacDist {
                        i: i.try_into().unwrap(),
                        j: j.try_into().unwrap(),
                        dist,
                    };
                    jac_dists.push(jac_dist);
                }
                {
                    let mut w = writer.lock().unwrap();
                    for jac_dist in jac_dists {
                        jac_dist.encode(w.deref_mut());
                    }
                }

                // TODO: External sort
            });
        }
        let duration = start.elapsed();
        eprintln!("Computed in {} sec", duration.as_secs_f64());
        possible_pairs
    };

    let radii = vec![0.1, 0.2, 0.5];
    let mut header = "num_chunks,dimensions,mean_absolute_error".to_string();
    for &r in &radii {
        write!(header, ",precision_{r}")?;
        write!(header, ",recall_{r}")?;
        write!(header, ",f1_{r}")?;
    }
    println!("{header}");

    eprintln!("Computing accuracy...");
    let start = Instant::now();

    for num_chunks in 1..=MAX_CHUNKS {
        eprintln!("Processed {}/{}...", num_chunks, MAX_CHUNKS);

        let mut sum_error = 0.;
        let mut true_results: Vec<_> = (0..radii.len()).map(|_| HashSet::new()).collect();
        let mut appx_results: Vec<_> = (0..radii.len()).map(|_| HashSet::new()).collect();

        let mut reader = BufReader::new(File::open(TMP_FILENAME)?);

        for i in 0..sketches.len() {
            let x = &sketches[i];
            for (j, y) in sketches.iter().enumerate().skip(i + 1) {
                let jac_dist = JacDist::decode(&mut reader).unwrap();
                assert_eq!(jac_dist.i, i.try_into().unwrap());
                assert_eq!(jac_dist.j, j.try_into().unwrap());

                let jac_dist = jac_dist.dist;
                let ham_dist = hamming_distance(&x[..num_chunks], &y[..num_chunks]);
                sum_error += (jac_dist - ham_dist).abs();

                for (k, &r) in radii.iter().enumerate() {
                    if jac_dist <= r {
                        true_results[k].insert((i, j));
                    }
                    if ham_dist <= r {
                        appx_results[k].insert((i, j));
                    }
                }
            }
        }

        let dim = num_chunks * 64;
        let mae = sum_error / possible_pairs as f64;

        let mut prf = vec![];
        for (tr, ar) in true_results.iter().zip(appx_results.iter()) {
            let true_positive = tr.intersection(ar).count() as f64;
            let false_positive = ar.len() as f64 - true_positive;
            let false_negative = tr.len() as f64 - true_positive;
            let precision = true_positive / (true_positive + false_positive);
            let recall = true_positive / (true_positive + false_negative);
            let f1 = (2. * precision * recall) / (precision + recall);
            prf.push((precision, recall, f1));
        }
        let mut body = format!("{num_chunks},{dim},{mae}");
        for (p, r, f) in prf {
            write!(body, ",{p},{r},{f}")?;
        }
        println!("{body}");
    }
    let duration = start.elapsed();
    eprintln!("Computed in {} sec", duration.as_secs_f64());

    Ok(())
}

fn hamming_distance(xs: &[u64], ys: &[u64]) -> f64 {
    assert_eq!(xs.len(), ys.len());
    let mut dist = 0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        dist += x.hamdist(y);
    }
    // In 1-bit minhash, the collision probability is multiplied by 2 over the original.
    // Thus, we should modify the Hamming distance with a factor of 2.
    dist as f64 / (xs.len() * 64) as f64 * 2.
}

#[derive(Debug, PartialEq, PartialOrd)]
struct JacDist {
    i: u32,
    j: u32,
    dist: f64,
}

impl JacDist {
    fn encode<W: Write>(&self, write: &mut W) {
        write.write_u32::<byteorder::LittleEndian>(self.i).unwrap();
        write.write_u32::<byteorder::LittleEndian>(self.j).unwrap();
        write
            .write_f64::<byteorder::LittleEndian>(self.dist)
            .unwrap();
    }

    fn decode<R: Read>(read: &mut R) -> Option<Self> {
        let i = read.read_u32::<byteorder::LittleEndian>().unwrap();
        let j = read.read_u32::<byteorder::LittleEndian>().unwrap();
        let dist = read.read_f64::<byteorder::LittleEndian>().unwrap();
        Some(Self { i, j, dist })
    }
}
