use crate::geometrics::{Point, Rectangle, Size};
use crate::tree::NodeId;

pub trait LayoutContext {
    fn get_rectangle(&self, node_id: NodeId) -> &Rectangle;

    fn get_rectangle_mut(&mut self, node_id: NodeId) -> &mut Rectangle;

    #[inline]
    fn get_point(&self, node_id: NodeId) -> &Point {
        &self.get_rectangle(node_id).point
    }

    #[inline]
    fn get_size(&self, node_id: NodeId) -> &Size {
        &self.get_rectangle(node_id).size
    }

    #[inline]
    fn arrange(&mut self, node_id: NodeId, point: Point) {
        (*self.get_rectangle_mut(node_id)).point = point;
    }
}

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
        max: Size::ZERO,
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
