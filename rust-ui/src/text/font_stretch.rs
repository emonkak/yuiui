use std::hash::Hash;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontStretch {
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

impl Default for FontStretch {
    fn default() -> Self {
        Self::Normal
    }
}
