use super::rect_outsets::RectOutsets;
use super::size::Size;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl BoxConstraints {
    pub const LOOSE: Self = Self {
        min: Size::ZERO,
        max: Size {
            width: f32::INFINITY,
            height: f32::INFINITY,
        },
    };

    #[inline]
    pub fn tight(size: Size) -> Self {
        Self {
            min: size,
            max: size,
        }
    }

    #[inline]
    pub fn constrain(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min.width, self.max.width),
            height: size.height.clamp(self.min.height, self.max.height),
        }
    }

    #[inline]
    pub fn deflate(&self, padding: RectOutsets) -> Self {
        Self {
            min: Size {
                width: self.min.width - (padding.left + padding.right),
                height: self.min.height - (padding.top + padding.bottom),
            },
            max: Size {
                width: self.max.width - (padding.left + padding.right),
                height: self.max.height - (padding.top + padding.bottom),
            },
        }
    }
}
