use super::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    Color(Color),
}

impl From<Color> for Background {
    #[inline]
    fn from(color: Color) -> Self {
        Background::Color(color)
    }
}
