use std::any::Any;

use geometrics::Rectangle;
use widget::{Element, Widget};
use window::x11::{XWindowHandle, XPaintContext};

#[derive(PartialEq, Eq)]
pub struct Fill {
    color: u32,
}

impl Fill {
    pub fn new(color: u32) -> Fill {
        Fill {
            color
        }
    }
}

impl Widget<XWindowHandle, XPaintContext> for Fill {
    fn paint(&mut self, handle: &XWindowHandle, rectangle: &Rectangle, paint_context: &mut XPaintContext) {
        paint_context.fill_rectangle(self.color, rectangle);
        paint_context.copy_to(handle.window, rectangle);
    }

    fn should_rerender(&self, next_widget: &Box<dyn Widget<XWindowHandle, XPaintContext>>, _next_children: &Box<[Element<XWindowHandle, XPaintContext>]>) -> bool {
        !self.same_widget(next_widget)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
