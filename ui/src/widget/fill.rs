use geometrics::{BoxConstraints, Rectangle, Size};
use graph::NodeId;
use ui::{LayoutContext, LayoutResult, UIState};
use widget::Widget;
use window::x11::{XWindowHandle, XPaintContext};

pub struct Fill {
    color: u32,
}

impl Fill {
    pub fn new(color: u32) -> Fill {
        Fill {
            color
        }
    }

    pub fn ui(self, context: &mut UIState<XWindowHandle, XPaintContext>) -> NodeId {
        context.add(self, &[])
    }
}

impl Widget<XWindowHandle, XPaintContext> for Fill {
    fn layout(
        &mut self,
        box_constraints: &BoxConstraints,
        _children: &[NodeId],
        _size: Option<Size>,
        _ctx: &mut LayoutContext
    ) -> LayoutResult {
        LayoutResult::Size(box_constraints.max)
    }

    fn connect(&mut self, parent_handle: &XWindowHandle, _rectangle: &Rectangle, _paint_context: &mut XPaintContext) -> XWindowHandle {
        parent_handle.clone()
    }

    fn paint(&mut self, handle: &XWindowHandle, rectangle: &Rectangle, paint_context: &mut XPaintContext) {
        paint_context.fill_rectangle(self.color, rectangle);
        paint_context.copy_to(handle.window, rectangle);
    }
}
