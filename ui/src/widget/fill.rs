use std::any::Any;

use geometrics::Rectangle;
use paint::PaintContext;
use widget::widget::{Element, Widget, WidgetMaker};
use window::x11::{XWindowHandle};

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

impl Widget<XWindowHandle> for Fill {
    fn paint(&mut self, handle: &XWindowHandle, rectangle: &Rectangle, paint_context: &mut PaintContext<XWindowHandle>) {
        paint_context.fill_rectangle(self.color, rectangle);
        paint_context.commit(handle, rectangle);
    }

    fn should_update(&self, element: &Element<XWindowHandle>) -> bool {
        !self.same_widget(&*element.instance)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WidgetMaker for Fill {
}
