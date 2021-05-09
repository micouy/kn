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
    InvalidArgValue(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("No path found.")]
    NoPathFound,

    #[error("Abbreviation is empty.")]
    EmptyAbbr,

    #[error("Arg contains invalid UTF-8.")]
    ArgInvalidUnicode,

    #[error("Abbreviation contains wildcard at the last place.")]
    WildcardAtLastPlace,

    #[error("")]
    CtrlC,
}
