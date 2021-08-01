use crate::geometrics::{Point, Size};
use crate::tree::NodeId;

#[derive(Debug)]
pub enum LayoutRequest {
    ArrangeChild(NodeId, Point),
    LayoutChild(NodeId, BoxConstraints),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl BoxConstraints {
    pub const ZERO: Self = Self {
        min: Size::ZERO,
        max: Size::ZERO,
    };

    #[inline]
    pub fn tight(size: &Size) -> BoxConstraints {
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
