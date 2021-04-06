use super::MatchStrength;

#[derive(Debug, Clone)]
pub struct Slice(pub String);

impl Slice {
    pub fn match_component(&self, component: &str) -> SliceMatch {
        use MatchStrength::*;

        if component.contains(&self.0) {
            if self.0 == component {
                SliceMatch::Yes(Complete)
            } else {
                SliceMatch::Yes(Partial)
            }
        } else {
            SliceMatch::No
        }
    }
}

// TODO: Think of better names for variants.
#[derive(Clone, Debug)]
pub enum SliceMatch {
    Yes(MatchStrength),
    No,
}
