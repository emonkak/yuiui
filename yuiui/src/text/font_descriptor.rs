use super::{Family, Stretch, Style, Weight};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct FontDescriptor {
    pub family: Family,
    pub style: Style,
    pub weight: Weight,
    pub stretch: Stretch,
}
