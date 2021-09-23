use std::hash::Hash;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

impl Default for Style {
    fn default() -> Self {
        Self::Normal
    }
}
