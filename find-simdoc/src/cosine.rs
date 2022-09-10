use crate::errors::{FindSimdocError, Result};
use crate::feature::{FeatureConfig, FeatureExtractor};
use crate::tfidf::{Idf, Tf};

use all_pairs_hamming::chunked_join::ChunkedJoiner;
use lsh::simhash::SimHasher;
use rand::{RngCore, SeedableRng};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum TfWeights {
    Binary,
    Standard,
    Sublinear,
}

#[derive(Clone, Debug)]
pub enum IdfWeights {
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

pub struct CosineSearcher {
    config: FeatureConfig,
    hasher: SimHasher,
    shows_progress: bool,
    tf_weight: TfWeights,
    idf_weight: IdfWeights,
    idf: Option<Idf<u64>>,
    joiner: Option<ChunkedJoiner<u64>>,
}

impl CosineSearcher {
    pub fn new(window_size: usize, delimiter: Option<char>, seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or_else(rand::random::<u64>);
        let mut seeder = rand_xoshiro::SplitMix64::seed_from_u64(seed);
        let config = FeatureConfig::new(window_size, delimiter, seeder.next_u64());
        let hasher = SimHasher::new(seeder.next_u64());
        Self {
            config,
            hasher,
            shows_progress: false,
            tf_weight: TfWeights::Binary,
            idf_weight: IdfWeights::Unary,
            idf: None,
            joiner: None,
        }
    }

    pub fn shows_progress(mut self, yes: bool) -> Self {
        self.shows_progress = yes;
        self
    }

    pub fn tf(mut self, tf_weight: TfWeights) -> Self {
        self.tf_weight = tf_weight;
        self
    }

    pub fn idf<I, D>(mut self, idf_weight: IdfWeights, documents: Option<I>) -> Result<Self>
    where
        I: IntoIterator<Item = D>,
        D: AsRef<str>,
    {
        match idf_weight {
            IdfWeights::Unary => {}
            IdfWeights::Standard | IdfWeights::Smooth => {
                if let Some(documents) = documents {
                    let mut extractor = FeatureExtractor::new(self.config);
                    let mut idf = Idf::new();
                    let mut feature = vec![];
                    for doc in documents {
                        let doc = doc.as_ref();
                        if doc.is_empty() {
                            return Err(FindSimdocError::input(
                                "Input document must not be empty.",
                            ));
                        }
                        extractor.extract(doc, &mut feature);
                        idf.add(&feature);
                    }
                    self.idf = Some(idf);
                } else {
                    return Err(FindSimdocError::input("Input document must not be empty."));
                }
            }
        }
        self.idf_weight = idf_weight;
        Ok(self)
    }

    pub fn build_sketches<I, D>(mut self, documents: I, num_chunks: usize) -> Self
    where
        I: IntoIterator<Item = D>,
        D: AsRef<str>,
    {
        let mut joiner = ChunkedJoiner::<u64>::new(num_chunks).shows_progress(self.shows_progress);
        let mut extractor = FeatureExtractor::new(self.config);
        let mut tf = Tf::new();
        let mut feature = vec![];
        for (i, doc) in documents.into_iter().enumerate() {
            if self.shows_progress && (i + 1) % 1000 == 0 {
                eprintln!("Processed {} documents...", i + 1);
            }
            let doc = doc.as_ref();
            assert!(!doc.is_empty());
            extractor.extract_with_weights(doc, &mut feature);
            match self.tf_weight {
                TfWeights::Binary => {}
                TfWeights::Standard => {
                    tf.tf(&mut feature);
                }
                TfWeights::Sublinear => {
                    tf.tf_sublinear(&mut feature);
                }
            }
            match self.idf_weight {
                IdfWeights::Unary => {}
                IdfWeights::Standard => {
                    let idf = self.idf.as_ref().unwrap();
                    for (term, weight) in feature.iter_mut() {
                        *weight *= idf.idf(*term);
                    }
                }
                IdfWeights::Smooth => {
                    let idf = self.idf.as_ref().unwrap();
                    for (term, weight) in feature.iter_mut() {
                        *weight *= idf.idf_smooth(*term);
                    }
                }
            }
            joiner.add(self.hasher.iter(&feature)).unwrap();
        }
        self.joiner = Some(joiner);
        self
    }

    pub fn search_similar_pairs(&self, radius: f64) -> Vec<(usize, usize, f64)> {
        self.joiner.as_ref().unwrap().similar_pairs(radius)
    }

    pub fn len(&self) -> usize {
        if let Some(joiner) = self.joiner.as_ref() {
            joiner.num_sketches()
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn memory_in_bytes(&self) -> usize {
        if let Some(joiner) = self.joiner.as_ref() {
            joiner.memory_in_bytes()
        } else {
            0
        }
    }
}
