use geometrics::{Size};
use tree::NodeId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

pub enum LayoutResult {
    Size(Size),
    RequestChild(NodeId, BoxConstraints),
}

impl BoxConstraints {
    pub fn tight(size: &Size) -> BoxConstraints {
        BoxConstraints {
            min: *size,
            max: *size,
        }
    }

    pub fn constrain(&self, size: &Size) -> Size {
        Size {
            width: size.width.clamp(self.min.width, self.max.width),
            height: size.height.clamp(self.min.height, self.max.height),
        }
    }
}
