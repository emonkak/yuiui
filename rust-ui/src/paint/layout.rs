use crate::geometrics::{Point, Size};
use crate::widget::WidgetId;

#[derive(Debug)]
pub enum LayoutRequest {
    ArrangeChild(WidgetId, Point),
    LayoutChild(WidgetId, BoxConstraints),
}

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
    pub fn tight(size: Size) -> BoxConstraints {
        let size = size.expand();
        BoxConstraints {
            min: size,
            max: size,
        }
    }

    #[inline]
    pub fn constrain(&self, size: &Size) -> Size {
        Size {
            width: size.width.clamp(self.min.width, self.max.width),
            height: size.height.clamp(self.min.height, self.max.height),
        }
    }
}
