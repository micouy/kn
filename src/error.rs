use thiserror::Error;

/// `kn` error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal error at {file}:{line}. Cause: {cause}. If you see this, contact the dev.")]
    DevError {
        line: u32,
        file: &'static str,
        cause: &'static str,
    },
    #[error(
        "Invalid slice. Slices should only contain alphanumeric characters."
    )]
    InvalidSlice,
    #[error("{0}")]
    IO(#[from] std::io::Error),
}

macro_rules! dev_err {
    ($cause:expr) => {
        Error::DevError {
            line: line!(),
            file: file!(),
            cause: $cause,
        }
    };
}
