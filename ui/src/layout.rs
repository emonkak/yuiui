use geometrics::Size;
use tree::NodeId;

#[derive(Debug)]
pub enum LayoutResult {
    Size(Size),
    RequestChild(NodeId, BoxConstraints),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl BoxConstraints {
    pub const NONE: Self = Self {
        min: Size::ZERO,
        max: Size::ZERO
    };

    #[inline]
    pub fn tight(size: Size) -> BoxConstraints {
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
