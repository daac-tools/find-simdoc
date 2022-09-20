#![allow(clippy::mutex_atomic)]

use std::env;
use std::error::Error;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::mem;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use all_pairs_hamming::sketch::Sketch;
use clap::Parser;
use find_simdoc::feature::{FeatureConfig, FeatureExtractor};
use find_simdoc::lsh::minhash::MinHasher;
use hashbrown::HashSet;
use positioned_io::WriteAt;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;

const MAX_CHUNKS: usize = 100;

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

    /// Directory path to write a tmp file.
    #[clap(short = 't', long)]
    tmp_dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let document_path = args.document_path;
    let delimiter = args.delimiter;
    let window_size = args.window_size;
    let seed = args.seed;
    let tmp_dir = args.tmp_dir;

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

        let processed = Mutex::new(0usize);

        let mut sketches = vec![vec![]; features.len()];
        features
            .par_iter()
            .map(|feature| {
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

    let tmp_path = {
        let mut tmp_path = tmp_dir.unwrap_or(env::temp_dir());
        tmp_path.push("tmp.jac_dist");
        tmp_path
    };

    let possible_pairs = {
        let start = Instant::now();

        let possible_pairs = features.len() * (features.len() - 1) / 2;
        eprintln!("Computing exact Jaccard distances for {possible_pairs} pairs...");

        let tmp_file_size = possible_pairs * mem::size_of::<f64>();
        let offsets = {
            let mut offset = 0;
            let mut offsets = Vec::with_capacity(features.len());
            for i in 0..features.len() {
                offsets.push(offset);
                offset += features.len() - i - 1;
            }
            assert_eq!(offset, possible_pairs);
            offsets
        };

        {
            let processed = Mutex::new(0usize);
            let writer = Mutex::new(File::create(&tmp_path)?);

            // Creates a file object of size tmp_file_size bytes.
            {
                let mut w = writer.lock().unwrap();
                w.write_at(tmp_file_size as u64 - 1, &[0])?;
            }

            eprintln!(
                "Created a tmp file of {} GiB, at {:?}",
                tmp_file_size as f64 / (1024. * 1024. * 1024.),
                &tmp_path
            );

            (0..features.len()).into_par_iter().for_each(|i| {
                {
                    // Mutex::lock also locks eprintln.
                    let mut cnt = processed.lock().unwrap();
                    *cnt += 1;
                    if *cnt % 1000 == 0 {
                        eprintln!("Processed {} features...", *cnt);
                    }
                }

                let mut jac_dists =
                    Vec::with_capacity((features.len() - i) * mem::size_of::<f64>());

                let x = &features[i];
                for y in features.iter().skip(i + 1) {
                    let dist =
                        find_simdoc::lsh::jaccard_distance(x.iter().clone(), y.iter().clone());
                    jac_dists.extend_from_slice(&dist.to_le_bytes());
                }

                // Writes distances with random access on a file stream.
                let offset = offsets[i] * mem::size_of::<f64>();
                {
                    let mut w = writer.lock().unwrap();
                    w.write_at(offset as u64, &jac_dists).unwrap();
                }
            });
        }

        let duration = start.elapsed();
        eprintln!("Computed in {} sec", duration.as_secs_f64());
        possible_pairs
    };

    let radii = vec![0.01, 0.02, 0.05, 0.1, 0.2, 0.5];
    let mut header = "num_chunks,dimensions,mean_absolute_error".to_string();
    for &r in &radii {
        write!(header, ",results_{r}")?;
        write!(header, ",precision_{r}")?;
        write!(header, ",recall_{r}")?;
        write!(header, ",f1_{r}")?;
    }
    println!("{header}");

    eprintln!("Computing accuracy...");
    let start = Instant::now();

    let results = {
        let processed = Mutex::new(0usize);
        let mut results: Vec<_> = (1..=MAX_CHUNKS)
            .into_par_iter()
            .map(|num_chunks| {
                {
                    // Mutex::lock also locks eprintln.
                    let mut cnt = processed.lock().unwrap();
                    *cnt += 1;
                    if *cnt % 10 == 0 {
                        eprintln!("Processed {} chunks...", *cnt);
                    }
                }

                let mut sum_error = 0.;
                let mut true_results: Vec<_> = (0..radii.len()).map(|_| HashSet::new()).collect();
                let mut appx_results: Vec<_> = (0..radii.len()).map(|_| HashSet::new()).collect();

                let mut reader = BufReader::new(File::open(&tmp_path).unwrap());

                for i in 0..sketches.len() {
                    let x = &sketches[i];
                    for (j, y) in sketches.iter().enumerate().skip(i + 1) {
                        let mut buf = [0; mem::size_of::<f64>()];
                        reader.read_exact(&mut buf).unwrap();

                        let jac_dist = f64::from_le_bytes(buf);
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
                    prf.push((tr.len(), precision, recall, f1));
                }

                let mut body = format!("{num_chunks},{dim},{mae}");
                for (t, p, r, f) in prf {
                    write!(body, ",{t},{p},{r},{f}").unwrap();
                }
                (num_chunks, body)
            })
            .collect();
        results.sort_by_key(|r| r.0);
        results
    };
    let duration = start.elapsed();
    eprintln!("Computed in {} sec", duration.as_secs_f64());

    for (_, body) in results {
        println!("{body}");
    }

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
