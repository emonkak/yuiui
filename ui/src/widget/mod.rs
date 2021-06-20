use geometrics::{BoxConstraints, Rectangle, Point, Size};
use graph::NodeId;
use ui::{LayoutContext, LayoutResult};

pub mod fill;
pub mod flex;
pub mod null;
pub mod padding;

pub trait Widget<WindowHandle, PaintContext> {
    fn layout(
        &mut self,
        bc: &BoxConstraints,
        children: &[NodeId],
        size: Option<Size>,
        layout_context: &mut LayoutContext
    ) -> LayoutResult {
        if let Some(size) = size {
            layout_context.position_child(children[0], Point { x: 0.0, y: 0.0 });
            LayoutResult::Size(size)
        } else {
            LayoutResult::RequestChild(children[0], *bc)
        }
    }

    fn connect(&mut self, parent_handle: &WindowHandle, rectangle: &Rectangle, paint_context: &mut PaintContext) -> WindowHandle;

    fn paint(&mut self, _handle: &WindowHandle, _rectangle: &Rectangle, _paint_context: &mut PaintContext) {
    }

    fn on_child_removed(&mut self, _child: NodeId) {
    }
}
