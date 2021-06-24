use std::any::Any;

use widget::Element;
use widget::Widget;

pub struct Null;

impl<WindowHandle, PaintContext> Widget<WindowHandle, PaintContext> for Null {
    fn should_rerender(&self, _next_widget: &Box<dyn Widget<WindowHandle, PaintContext>>, _next_children: &Box<[Element<WindowHandle, PaintContext>]>) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

