use super::color::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    Color(Color),
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Color(color)
    }
}

impl From<Color> for Option<Background> {
    fn from(color: Color) -> Self {
        Some(Background::from(color))
    }
}
