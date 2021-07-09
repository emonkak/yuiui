use geometrics::{Point, Rectangle, Size};
use tree::{NodeId, Tree};

pub trait Layout<Widget> {
    fn measure(
        &mut self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &Tree<Widget>,
        _layouter: &mut dyn Layouter
    ) -> LayoutResult;
}

pub trait Layouter {
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

pub struct DefaultLayout;

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

impl<Widget> Layout<Widget> for DefaultLayout {
    fn measure(
        &mut self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &Tree<Widget>,
        _layouter: &mut dyn Layouter
    ) -> LayoutResult {
        if let Some((_, size)) = response {
            LayoutResult::Size(size)
        } else {
            if let Some(child_id) = tree[node_id].first_child() {
                LayoutResult::RequestChild(child_id, box_constraints)
            } else {
                LayoutResult::Size(box_constraints.max)
            }
        }
    }
}
