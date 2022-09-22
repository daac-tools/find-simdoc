//! Error definitions.
use std::error::Error;
use std::{fmt, result};

/// A specialized Result type for this library.
pub type Result<T, E = AllPairsHammingError> = result::Result<T, E>;

/// Errors in this library.
#[derive(Debug)]
pub enum AllPairsHammingError {
    /// Contains [`InputError`].
    Input(InputError),
}

impl fmt::Display for AllPairsHammingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Input(e) => e.fmt(f),
        }
    }
}

impl Error for AllPairsHammingError {}

impl AllPairsHammingError {
    pub(crate) const fn input(msg: String) -> Self {
        Self::Input(InputError { msg })
    }
}

/// Error used when the input argument is invalid.
#[derive(Debug)]
pub struct InputError {
    msg: String,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InputError: {}", self.msg)
    }
}
