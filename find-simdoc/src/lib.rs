//! Fast all-pair similarity searches in documents.
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
//! - **Easy to use:** This software supports all essential steps of document similarity search,
//! from feature extraction to output of similar pairs.
//! Therefore, you can immediately try the fast all-pair similarity search using your document files.
//!
//! - **Flexible tokenization:** You can specify any delimiter when splitting words in tokenization for feature extraction.
//! This can be useful in languages where multiple definitions of words exist, such as Japanese or Chinese.
//!
//! - **Time- and memory-efficient:** The time complexity is *linear* over the numbers of input documents and output results,
//! based on the idea of the [sketch sorting approach](https://proceedings.mlr.press/v13/tabei10a.html).
//! The memory complexity is *linear* over the numbers of input documents,
//! and the actual memory usage is also very low thanks to locality sensitive hashing.
//!
//! - **Pure Rust:** This software is implemented in Rust, achieving safe and fast performance.
//!
//! # Search steps
//!
//! 1. Extract features from documents
//!    - Set representation of character or word ngrams
//!    - Tfidf-weighted vector representation of character or word ngrams
//! 2. Convert the features into binary sketches through locality sensitive hashing (LSH)
//!    - [1-bit minwise hashing](https://arxiv.org/abs/0910.3349) for the Jaccard similarity
//!    - [Simplified simhash](https://dl.acm.org/doi/10.1145/1242572.1242592) for the Cosine similarity
//! 3. Search for similar sketches in the Hamming space using a modified variant of the [sketch sorting approach](https://proceedings.mlr.press/v13/tabei10a.html)
#![deny(missing_docs)]

pub mod cosine;
pub mod errors;
pub mod feature;
pub mod jaccard;
pub mod tfidf;

pub(crate) mod shingling;

pub use cosine::CosineSearcher;
pub use jaccard::JaccardSearcher;
