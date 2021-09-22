//! Utils.

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
/// let distance_1 = assert_variant!(abbr.compare("abc_jkl"), Some(Subsequence(distance)) => distance);
/// let distance_2 = assert_variant!(abbr.compare("ab_cj_kl"), Some(Subsequence(distance)) => distance);
/// assert!(distance_1 < distance_2);
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
