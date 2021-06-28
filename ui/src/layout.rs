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

impl LayoutContext {
    pub(crate) const fn new() -> Self {
        Self {
            rectangles: SlotVec::new(),
        }
    }

    #[inline]
    pub(crate) fn remove(&mut self, node_id: NodeId) -> Rectangle {
        self.rectangles.remove(node_id)
    }

    #[inline]
    pub fn get_rectangle(&self, node_id: NodeId) -> Option<&Rectangle> {
        self.rectangles.get(node_id)
    }

    #[inline]
    pub fn get_point(&self, node_id: NodeId) -> Option<&Point> {
        self.rectangles
            .get(node_id)
            .map(|rectangle| &rectangle.point)
    }

    #[inline]
    pub fn get_size(&self, node_id: NodeId) -> Option<&Size> {
        self.rectangles
            .get(node_id)
            .map(|rectangle| &rectangle.size)
    }

    #[inline]
    pub fn arrange(&mut self, node_id: NodeId, point: Point) {
        let rectange = self.rectangles.get_or_insert_default(node_id);
        rectange.point = point;
    }

    #[inline]
    pub fn resize(&mut self, node_id: NodeId, size: Size) {
        let rectange = self.rectangles.get_or_insert_default(node_id);
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
