//! Abbreviations.

use std::cmp::{Ord, Ordering};

use powierza_coefficient::powierża_coefficient;

/// A component of the user's query.
///
/// It is used in comparing and ordering of found paths. Read more in
/// [`Congruence`'s docs](Congruence).
#[derive(Debug, Clone)]
pub enum Abbr {
    /// Wildcard matches every component with congruence
    /// [`Complete`](Congruence::Complete).
    Wildcard,

    /// Literal abbreviation.
    Literal(String),
}

impl Abbr {
    /// Constructs [`Abbr::Wildcard`](Abbr::Wildcard) if the
    /// string slice is '-', otherwise constructs
    /// wrapped [`Abbr::Literal`](Abbr::Literal) with the abbreviation
    /// mapped to its ASCII lowercase equivalent.
    pub fn new_sanitized(abbr: &str) -> Self {
        if abbr == "-" {
            Self::Wildcard
        } else {
            Self::Literal(abbr.to_ascii_lowercase())
        }
    }

    /// Compares a component against the abbreviation.
    pub fn compare(&self, component: &str) -> Option<Congruence> {
        // What about characters with accents? [https://eev.ee/blog/2015/09/12/dark-corners-of-unicode/]
        let component = component.to_ascii_lowercase();

        match self {
            Self::Wildcard => Some(Congruence::Complete),
            Self::Literal(literal) =>
                if literal.is_empty() || component.is_empty() {
                    None
                } else if *literal == component {
                    Some(Congruence::Complete)
                } else if component.starts_with(literal) {
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
    /// characters. The field contains the Powierża coefficient of the pair of
    /// strings.
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
        let abbr = Abbr::new_sanitized("abcjkl");

        assert_variant!(abbr.compare("abcjkl"), Some(Complete));
        assert_variant!(abbr.compare("abcjkl_"), Some(Prefix));
        assert_variant!(abbr.compare("_abcjkl"), Some(Subsequence(0)));
        assert_variant!(abbr.compare("abc_jkl"), Some(Subsequence(1)));

        assert_variant!(abbr.compare("xyz"), None);
        assert_variant!(abbr.compare(""), None);
    }

    #[test]
    fn test_compare_abbr_different_cases() {
        let abbr = Abbr::new_sanitized("AbCjKl");

        assert_variant!(abbr.compare("aBcJkL"), Some(Complete));
        assert_variant!(abbr.compare("AbcJkl_"), Some(Prefix));
        assert_variant!(abbr.compare("_aBcjKl"), Some(Subsequence(0)));
        assert_variant!(abbr.compare("abC_jkL"), Some(Subsequence(1)));
    }

    #[test]
    fn test_empty_abbr_empty_component() {
        let empty = "";

        let abbr = Abbr::new_sanitized(empty);
        assert_variant!(abbr.compare("non empty component"), None);

        let abbr = Abbr::new_sanitized("non empty abbr");
        assert_variant!(abbr.compare(empty), None);
    }

    #[test]
    fn test_order_paths() {
        fn sort<'a>(paths: &'a Vec<&'a str>, abbr: &str) -> Vec<&'a str> {
            let abbr = Abbr::new_sanitized(abbr);
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
