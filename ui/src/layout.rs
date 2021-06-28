use std::ops::{Index, IndexMut};

use geometrics::{Point, Rectangle, Size};
use slot_vec::SlotVec;
use tree::NodeId;

#[derive(Debug)]
pub struct LayoutContext {
    rectangles: SlotVec<Rectangle>,
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

impl Index<NodeId> for LayoutContext {
    type Output = Rectangle;

    #[inline]
    fn index(&self, node_id: NodeId) -> &Self::Output {
        &self.rectangles[node_id]
    }
}

impl IndexMut<NodeId> for LayoutContext {
    #[inline]
    fn index_mut(&mut self, node_id: NodeId) -> &mut Self::Output {
        &mut self.rectangles[node_id]
    }
}

impl LayoutContext {
    pub(crate) const fn new() -> Self {
        Self {
            rectangles: SlotVec::new(),
        }
    }

    #[inline]
    pub(crate) fn insert_at(&mut self, node_id: NodeId, rectangle: Rectangle) {
        self.rectangles.insert_at(node_id, rectangle);
    }

    #[inline]
    pub(crate) fn remove(&mut self, node_id: NodeId) -> Rectangle {
        self.rectangles.remove(node_id)
    }

    #[inline]
    pub fn get_point(&self, node_id: NodeId) -> &Point {
        &self.rectangles[node_id].point
    }

    #[inline]
    pub fn get_size(&self, node_id: NodeId) -> &Size {
        &self.rectangles[node_id].size
    }

    #[inline]
    pub fn arrange(&mut self, node_id: NodeId, point: Point) {
        let rectange = &mut self.rectangles[node_id];
        rectange.point = point;
    }

    #[inline]
    pub fn resize(&mut self, node_id: NodeId, size: Size) {
        let rectange = &mut self.rectangles[node_id];
        rectange.size = size;
    }
}

impl BoxConstraints {
    #[inline]
    pub fn tight(size: &Size) -> BoxConstraints {
        BoxConstraints {
            min: *size,
            max: *size,
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
