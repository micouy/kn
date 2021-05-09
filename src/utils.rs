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

#[cfg(test)]
macro_rules! assert_has_elem {
    ($vec:expr , $( $pattern:pat )|+ $( if $guard: expr )?) => {{
        let any = $vec.iter().any(|elem| match elem {
            $( $pattern )|+ $( if $guard )? => { true },
            _ => false,
        });

        if !any {
            panic!("assertion failed");
        }
    }};

    ($vec:expr , $( $pattern:pat )|+ $( if $guard: expr )? => $expression_out:expr) => {{
        let mb_elem = $vec.iter().find_map(|elem| match elem {
            $( $pattern )|+ $( if $guard )? => { Some($expression_out) },
            _ => None,
        });

        match mb_elem {
            Some(elem) => elem,
            None => panic!("assertion failed"),
        }
    }};
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
