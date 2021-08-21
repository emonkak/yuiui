use crate::base::{Point, Size};
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