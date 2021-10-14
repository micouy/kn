//! Error.

use thiserror::Error;

/// Error.
#[derive(Error, Debug)]
pub enum Error {
    /// Wrapper around [`std::io::Error`](std::io::Error).
    #[error("IO error `{0}`.")]
    IO(#[from] std::io::Error),

    /// Non-Unicode input received.
    #[error("Non-Unicode input received.")]
    NonUnicodeInput,

    /// Path not found.
    #[error("Path not found.")]
    PathNotFound,

    /// An invalid arg value.
    #[error("Value of arg `{0}` is invalid.")]
    InvalidArgValue(String),

    /// Wrapper around [`pico_args::Error`](pico_args::Error).
    #[error("Args error: `{0}`.")]
    Args(#[from] pico_args::Error),

    /// Unexpected abbr component.
    #[error("Unexpected abbr component `{0}`.")]
    UnexpectedAbbrComponent(String),
}
