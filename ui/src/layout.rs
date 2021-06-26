use geometrics::{Size};
use tree::NodeId;
use fiber::{RenderingTree, RenderingNode};

pub type LayoutContext<'a, Window> = &'a mut RenderingTree<Window>;

pub type LayoutNode<Window> = RenderingNode<Window>;

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
