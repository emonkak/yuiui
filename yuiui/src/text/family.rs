use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Family {
    Name(String),
    SansSerif,
    Serif,
    Monospace,
}

impl Default for Family {
    fn default() -> Self {
        Self::SansSerif
    }
}
