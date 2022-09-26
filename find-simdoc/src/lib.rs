//! Time- and memory-efficient all pairs similarity searches in documents.
//! A more detailed description can be found on the [project page](https://github.com/legalforce-research/find-simdoc).
//!
//! # Problem definition
//!
//! - Input
//!   - List of documents
//!   - Distance function
//!   - Radius threshold
//! - Output
//!   - All pairs of similar document ids
//!
//! # Features
//!
//! ## Easy to use
//!
//! This software supports all essential steps of document similarity search,
//! from feature extraction to output of similar pairs.
//! Therefore, you can immediately try the fast all pairs similarity search using your document files.
//!
//! ## Flexible tokenization
//!
//! You can specify any delimiter when splitting words in tokenization for feature extraction.
//! This can be useful in languages where multiple definitions of words exist, such as Japanese or Chinese.
//!
//! ## Time and memory efficiency
//!
//! The time and memory complexities are *linear* over the numbers of input documents and output results
//! on the basis of the ideas behind the locality sensitive hashing (LSH) and [sketch sorting approach](https://proceedings.mlr.press/v13/tabei10a.html).
//!
//! ## Tunable search performance
//!
//! LSH allows tuning of performance in accuracy, time, and memory, through a manual parameter specifying search dimensions.
//! You can flexibly perform searches depending on your dataset and machine environment.
//!   - Specifying lower dimensions allows for faster and rougher searches with less memory usage.
//!   - Specifying higher dimensions allows for more accurate searches with more memory usage.
//!
//! # Search steps
//!
//! 1. Extract features from documents
//!    - Set representation of character or word ngrams
//!    - Tfidf-weighted vector representation of character or word ngrams
//! 2. Convert the features into binary sketches through locality sensitive hashing
//!    - [1-bit minwise hashing](https://dl.acm.org/doi/abs/10.1145/1772690.1772759) for the Jaccard similarity
//!    - [Simplified simhash](https://dl.acm.org/doi/10.1145/1242572.1242592) for the Cosine similarity
//! 3. Search for similar sketches in the Hamming space using a modified variant of the [sketch sorting approach](https://proceedings.mlr.press/v13/tabei10a.html)
#![deny(missing_docs)]

pub mod cosine;
pub mod errors;
pub mod feature;
pub mod jaccard;
pub mod lsh;
pub mod tfidf;

mod shingling;

pub use cosine::CosineSearcher;
pub use jaccard::JaccardSearcher;
