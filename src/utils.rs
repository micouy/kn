//! Utils.

use std::{convert::AsRef, path::Path};

/// Asserts that the expression matches the variant. Optionally returns a value.
///
/// Inspired by [`std::matches`](https://doc.rust-lang.org/stable/std/macro.matches.html).
///
/// # Examples
///
/// ```
/// # fn main() -> Option<()> {
/// use kn::Congruence::*;
///
/// let abbr = Abbr::literal("abcjkl")?;
/// let coeff_1 = assert_variant!(abbr.compare("abc_jkl"), Some(Subsequence(coeff)) => coeff);
/// let coeff_2 = assert_variant!(abbr.compare("ab_cj_kl"), Some(Subsequence(coeff)) => coeff);
/// assert!(coeff_1 < coeff_2);
/// # Ok(())
/// # }
/// ```
#[cfg(any(test, doc))]
#[macro_export]
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

pub fn as_path<P>(path: &P) -> &Path
where
    P: AsRef<Path> + ?Sized,
{
    path.as_ref()
}
