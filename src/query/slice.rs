use super::MatchStrength;
use crate::{Error, Result};

use regex::Regex;

#[derive(Debug, Clone)]
pub enum Slice {
    Literal(String),
    Wildcard,
}

impl Slice {
    pub fn from_string(pattern: String) -> Result<Self> {
        let only_valid_re = Regex::new(r"^[\-_.a-zA-Z0-9]+$").unwrap();
        let only_dots_re = Regex::new(r"^\.+$").unwrap();

        if pattern == "-" {
            Ok(Self::Wildcard)
        } else {
            if pattern.is_empty() {
                return Err(Error::InvalidSlice(pattern));
            }
            if !only_valid_re.is_match(&pattern) {
                return Err(Error::InvalidSlice(pattern));
            }
            if only_dots_re.is_match(&pattern) {
                return Err(Error::InvalidSlice(pattern));
            }

            Ok(Self::Literal(pattern.to_ascii_lowercase()))
        }
    }

    // TODO: Make it return `MatchStrength`.
    pub fn match_component(&self, component: &str) -> SliceMatch {
        use MatchStrength::*;

        let component = component.to_ascii_lowercase();

        match self {
            Self::Literal(string) =>
                if component.contains(string) {
                    if string == &component {
                        SliceMatch::Yes(Complete)
                    } else {
                        SliceMatch::Yes(Partial)
                    }
                } else {
                    SliceMatch::No
                },
            Self::Wildcard => SliceMatch::Yes(Partial),
        }
    }
}

// TODO: Think of better names for variants.
#[derive(Clone, Debug)]
pub enum SliceMatch {
    Yes(MatchStrength),
    No,
}

#[cfg(test)]
mod test {
    use super::*;
    use MatchStrength::*;
    use SliceMatch::*;

    #[test]
    fn test_from_string() {
        let slice = |s: &str| Slice::from_string(s.to_string());

        assert!(slice("").is_err());
        assert!(slice(".").is_err());
        assert!(slice("..").is_err());
        assert!(slice("...").is_err());
        assert!(slice("one two three").is_err());

        let slice = |s: &str| Slice::from_string(s.to_string()).unwrap();

        variant!(slice("-"), Slice::Wildcard);
        variant!(slice("--"), Slice::Literal(literal) if literal == "--");
        variant!(slice("mOcKiNgBiRd"), Slice::Literal(literal) if literal == "mockingbird");
        variant!(slice("X.ae.A-12"), Slice::Literal(literal) if literal == "x.ae.a-12");

        // TODO
        // assert!(variant!(slice("zażółć"), Slice::Literal(literal) => literal
        // == "zażółć"));
    }

    #[test]
    fn test_wildcard_match() {
        let slice = Slice::Wildcard;

        variant!(slice.match_component("iks"), Yes(Partial));
        variant!(slice.match_component("eh--ehe123"), Yes(Partial));
        variant!(slice.match_component("coStaMm"), Yes(Partial));
    }

    #[test]
    fn test_literal_match() {
        let slice = Slice::Literal("mi".to_string());

        variant!(slice.match_component("mi"), Yes(Complete));
        variant!(slice.match_component("Mi"), Yes(Complete));
        variant!(slice.match_component("ooo..oo---mi-ooooo"), Yes(Partial));
        variant!(slice.match_component("ooo..oo---mI-ooooo"), Yes(Partial));
        variant!(slice.match_component("xxxxxx"), No);
    }
}
