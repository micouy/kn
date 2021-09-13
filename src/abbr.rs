//! Abbreviations.

use std::cmp::{Ord, Ordering};

/// The strength of the match between an abbreviation and a component.
///
/// [`Congruence`](Congruence) is used to order found paths in the following way:
///
/// 1. Path's components are compared from the "left" (parent then child).
/// 2. Components are first ordered based on how well they match the abbreviation — [`Full`](Congruence::Full)
///    then [`Prefix`](Congruence::Prefix) then [`Subseries`](Congruence::Subseries) then [`Wildcard`](Congruence::Wildcard).
/// 3. Multiple components with the same match strength, either [`Prefix`](Congruence::Prefix) or [`Subseries`](Congruence::Subseries),
///    are ordered according to the Levenshtein distance between the component and the abbreviation (smaller then greater).
/// 4. If the order of two components cannot be resolved based on the above, [`alphanumeric_sort`](https://docs.rs/alphanumeric-sort) is used.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Congruence {
    /// The abbreviation and the component are the same.
    Full,
    /// The abbreviation is a prefix of the component. The field contains the Levenshtein distance between them.
    Prefix(usize),
    /// The abbreviation is a subseries of the component. The field contains the Levenshtein distance between them.
    Subseries(usize),
    /// The abbreviation is a wildcard.
    Wildcard,
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
            (Full, Full) => Equal,
            (Full, _) => Less,

            (Prefix(_), Full) => Greater,
            (Prefix(dist_a), Prefix(dist_b)) => dist_a.cmp(dist_b),
            (Prefix(_), _) => Less,

            (Subseries(_), Full) => Greater,
            (Subseries(_), Prefix(_)) => Greater,
            (Subseries(dist_a), Subseries(dist_b)) => dist_a.cmp(dist_b),
            (Subseries(_), _) => Less,

			(Wildcard, Wildcard) => Equal,
			(Wildcard, _) => Greater,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
	fn test_congruence_ordering() {
        assert!(Full < Prefix(1));
        assert!(Full < Subseries(1));
        assert!(Full < Wildcard);

        assert!(Prefix(1) < Prefix(1000));
        assert!(Prefix(1000) < Subseries(1));
        assert!(Prefix(1000) < Wildcard);

        assert!(Subseries(1) < Subseries(1000));
        assert!(Subseries(1000) < Wildcard);
	}
}
