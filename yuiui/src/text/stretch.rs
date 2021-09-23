use std::hash::Hash;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Stretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl Default for Stretch {
    fn default() -> Self {
        Self::Normal
    }
}
