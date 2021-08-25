use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FontFamily {
    Name(String),
    SansSerif,
    Serif,
    Monospace,
}

impl Default for FontFamily {
    fn default() -> Self {
        Self::SansSerif
    }
}
