use std::error::Error;
use std::{fmt, result};

/// A specialized Result type for this library.
pub type Result<T, E = FindSimdocError> = result::Result<T, E>;

/// Errors in crawdad.
#[derive(Debug)]
pub enum FindSimdocError {
    /// Contains [`InputError`].
    Input(InputError),
}

impl fmt::Display for FindSimdocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Input(e) => e.fmt(f),
        }
    }
}

impl Error for FindSimdocError {}

impl FindSimdocError {
    pub(crate) const fn input(msg: &'static str) -> Self {
        Self::Input(InputError { msg })
    }
}

/// Error used when the input argument is invalid.
#[derive(Debug)]
pub struct InputError {
    msg: &'static str,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InputError: {}", self.msg)
    }
}
