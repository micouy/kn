//! Error.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// `kn` error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal error at {file}:{line}. Cause:\n{cause:#?}\nIf you see this, contact the dev.")]
    DevError {
        line: u32,
        file: &'static str,
        cause: Box<dyn std::fmt::Debug>,
    },

    #[error("Abbreviation `{0}` is invalid.")]
    InvalidAbbr(String),

    #[error("Value of arg `{0}` is invalid.")]
    InvalidArgumentValue(String),

    #[error(transparent)]
    Args(#[from] pico_args::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("No path found.")]
    NoPathFound,

    #[error("Invalid UTF-8 encountered.")]
    InvalidUnicode,

    #[error("")]
    CtrlC,
}
