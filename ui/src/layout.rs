use geometrics::{Point, Rectangle, Size};
use slot_vec::SlotVec;
use tree::NodeId;

#[derive(Debug)]
pub struct LayoutContext<'a> {
    states: &'a mut SlotVec<LayoutState>,
}

#[derive(Debug)]
pub struct LayoutState {
    pub rectangle: Rectangle,
    pub deleted_children: Vec<NodeId>,
}

pub enum LayoutResult {
    Size(Size),
    RequestChild(NodeId, BoxConstraints),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            rectangle: Rectangle::ZERO,
            deleted_children: Vec::new(),
        }
    }
}

impl<'a> LayoutContext<'a> {
    pub fn new(states: &'a mut SlotVec<LayoutState>) -> Self {
        Self {
            states
        }
    }

    #[inline]
    pub fn get_rectangle(&self, node_id: NodeId) -> &Rectangle {
        &self.states[node_id].rectangle
    }

    #[inline]
    pub fn get_point(&self, node_id: NodeId) -> &Point {
        &self.states[node_id].rectangle.point
    }

    #[inline]
    pub fn get_size(&self, node_id: NodeId) -> &Size {
        &self.states[node_id].rectangle.size
    }

    #[inline]
    pub fn arrange(&mut self, node_id: NodeId, point: Point) {
        self.states[node_id].rectangle.point = point;
    }
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

