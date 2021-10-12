//! Abbreviations.

use std::{
    cmp::{Ord, Ordering},
    str::pattern::Pattern,
};

use powierza_coefficient::powierża_coefficient;

/// A wrapper type around [`AbbrInner`](AbbrInner) exposing only safe
/// constructors.
#[derive(Debug, Clone)]
pub struct Abbr {
    inner: AbbrInner,
}

impl Abbr {
    /// Constructs wrapped [`AbbrInner::Wildcard`](AbbrInner::Wildcard) if the
    /// string slice is '-', otherwise constructs
    /// wrapped [`AbbrInner::Literal`](AbbrInner::Literal) with the abbreviation
    /// mapped to its ASCII lowercase equivalent.
    pub fn from_str(abbr: &str) -> Self {
        if abbr == "-" {
            Self {
                inner: AbbrInner::Wildcard,
            }
        } else {
            Self {
                inner: AbbrInner::Literal(abbr.to_ascii_lowercase()),
            }
        }
    }

    /// Compares a component against the abbreviation.
    pub fn compare(&self, component: &str) -> Option<Congruence> {
        self.inner.compare(component)
    }
}

/// A component of the user's query.
///
/// It is used in comparing and ordering of found paths. Read more in
/// [`Congruence`'s docs](Congruence).
#[derive(Debug, Clone)]
enum AbbrInner {
    /// Wildcard matches every component with congruence
    /// [`Complete`](Congruence::Complete).
    Wildcard,

    /// Literal abbreviation.
    Literal(String),
}

impl AbbrInner {
    /// Compares a component against the abbreviation.
    fn compare(&self, component: &str) -> Option<Congruence> {
        // What about characters with accents? [https://eev.ee/blog/2015/09/12/dark-corners-of-unicode/]
        let component = component.to_ascii_lowercase();

        match self {
            AbbrInner::Wildcard => Some(Congruence::Complete),
            AbbrInner::Literal(literal) =>
                if *literal == component {
                    Some(Congruence::Complete)
                } else if literal.is_prefix_of(&component) {
                    Some(Congruence::Prefix)
                } else {
                    powierża_coefficient(literal, &component)
                        .map(Congruence::Subsequence)
                },
        }
    }
}

/// The strength of the match between an abbreviation and a component.
///
/// [`Congruence`](Congruence) is used to order path components in the following
/// way:
///
/// 1. Components are first ordered based on how well they match the
/// abbreviation — first [`Complete`](Congruence::Complete), then
/// [`Prefix`](Congruence::Prefix), then
/// [`Subsequence`](Congruence::Subsequence).
/// 2. Components with congruence [`Subsequence`](Congruence::Subsequence) are
/// ordered by their [Powierża coefficient](https://github.com/micouy/powierza-coefficient).
/// 3. If the order of two components cannot be determined based on the above, [`alphanumeric_sort`](https://docs.rs/alphanumeric-sort) is used.
///
/// Below are the results of matching components against abbreviation `abc`:
///
/// | Component   | Match strength                           |
/// |-------------|------------------------------------------|
/// | `abc`       | [`Complete`](Congruence::Complete)       |
/// | `abc___`    | [`Prefix`](Congruence::Prefix)           |
/// | `_a_b_c_`   | [`Subsequence`](Congruence::Subsequence) |
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Congruence {
    /// Either the abbreviation and the component are the same or the
    /// abbreviation is a wildcard.
    Complete,

    /// The abbreviation is a prefix of the component.
    Prefix,

    /// The abbreviation's characters form a subsequence of the component's
    /// characters. The field contains the Powierża coefficient of the pair of strings.
    Subsequence(u32),
}

use Congruence::*;

impl PartialOrd for Congruence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for Congruence {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        match (self, other) {
            (Complete, Complete) => Equal,
            (Complete, Prefix) => Less,
            (Complete, Subsequence(_)) => Less,

            (Prefix, Complete) => Greater,
            (Prefix, Prefix) => Equal,
            (Prefix, Subsequence(_)) => Less,

            (Subsequence(_), Complete) => Greater,
            (Subsequence(_), Prefix) => Greater,
            (Subsequence(dist_a), Subsequence(dist_b)) => dist_a.cmp(dist_b),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_congruence_ordering() {
        assert!(Complete < Prefix);
        assert!(Complete < Subsequence(1));
        assert!(Prefix < Subsequence(1));
        assert!(Subsequence(1) < Subsequence(1000));
    }

    #[test]
    fn test_compare_abbr() {
        let abbr = Abbr::from_str("abcjkl");

        assert_variant!(abbr.compare("abcjkl"), Some(Complete));
        assert_variant!(abbr.compare("abcjkl_"), Some(Prefix));
        assert_variant!(abbr.compare("_abcjkl"), Some(Subsequence(0)));
        assert_variant!(abbr.compare("abc_jkl"), Some(Subsequence(1)));

        assert_variant!(abbr.compare("xyz"), None);
        assert_variant!(abbr.compare(""), None);
    }

    #[test]
    fn test_compare_abbr_different_cases() {
        let abbr = Abbr::from_str("AbCjKl");

        assert_variant!(abbr.compare("aBcJkL"), Some(Complete));
        assert_variant!(abbr.compare("AbcJkl_"), Some(Prefix));
        assert_variant!(abbr.compare("_aBcjKl"), Some(Subsequence(0)));
        assert_variant!(abbr.compare("abC_jkL"), Some(Subsequence(1)));
    }

    #[test]
    fn test_order_paths() {
        fn sort<'a>(paths: &'a Vec<&'a str>, abbr: &str) -> Vec<&'a str> {
            let abbr = Abbr::from_str(abbr);
            let mut paths = paths.clone();
            paths.sort_by_key(|path| abbr.compare(path).unwrap());

            paths
        }

        let paths = vec!["playground", "plotka"];
        assert_eq!(paths, sort(&paths, "pla"));

        let paths = vec!["veccentric", "vehiccles"];
        assert_eq!(paths, sort(&paths, "vecc"));
    }
}
