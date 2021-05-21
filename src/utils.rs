use std::{convert::AsRef, path::Path};

pub fn as_path<P>(path: &P) -> &Path
where
    P: AsRef<Path> + ?Sized,
{
    path.as_ref()
}

// Inspired by `https://doc.rust-lang.org/stable/std/macro.matches.html`.
#[cfg(test)]
macro_rules! assert_variant {
    ($expression_in:expr , $( $pattern:pat )|+ $( if $guard: expr )? $( => $expression_out:expr )? ) => {
        match $expression_in {
            $( $pattern )|+ $( if $guard )? => { $( $expression_out )? },
            variant => panic!("{:?}", variant),
        }
    };


    ($expression_in:expr , $( $pattern:pat )|+ $( if $guard: expr )? $( => $expression_out:expr)? , $panic:expr) => {
        match $expression_in {
            $( $pattern )|+ $( if $guard )? => { $( $expression_out )? },
            _ => panic!($panic),
        }
    };
}

macro_rules! dev_err {
    ($cause:expr) => {
        Error::DevError {
            line: line!(),
            file: file!(),
            cause: Box::new($cause),
        }
    };
    () => {
        Error::DevError {
            line: line!(),
            file: file!(),
            cause: Box::new(None::<()>),
        }
    };
}
