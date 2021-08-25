use std::hash::Hash;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Normal
    }
}
