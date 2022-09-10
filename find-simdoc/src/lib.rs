//! Fast all-pair similarity searches in documents.
#![deny(missing_docs)]

pub mod cosine;
pub mod errors;
pub mod feature;
pub mod jaccard;
pub mod tfidf;

pub(crate) mod shingling;

pub use cosine::CosineSearcher;
pub use jaccard::JaccardSearcher;
