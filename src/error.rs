use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error.")]
    IO(#[from] std::io::Error),

    #[error("Non-Unicode input received.")]
    NonUnicodeInput,

    #[error("Path not found.")]
    PathNotFound,

    #[error("Value of arg `{0}` is invalid.")]
    InvalidArgValue(String),

    #[error(transparent)]
    Args(#[from] pico_args::Error),
}
