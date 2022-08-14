use crate::geometrics::RectOutsets;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutStyle {
    pub flex: f32,
    pub width: Length,
    pub height: Length,
    pub padding: RectOutsets,
    pub margin: RectOutsets,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Auto,
    Pixel(f32),
}

impl Default for Length {
    fn default() -> Self {
        Self::Auto
    }
}
