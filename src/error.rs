//! Error.

use thiserror::Error;

/// `kn` error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal error at {file}:{line}. Cause:\n{cause}\nIf you see this, contact the dev.")]
    DevError {
        line: u32,
        file: &'static str,
        cause: String,
    },
    #[error("Invalid slice: `{0}`.")]
    InvalidSlice(String),
    #[error("Invalid value for arg `{0}`.")]
    InvalidArgValue(String),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("No path found.")]
    NoPathFound,
}
