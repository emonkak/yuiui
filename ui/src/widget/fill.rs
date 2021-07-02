use std::any::Any;

use geometrics::Rectangle;
use paint::PaintContext;
use widget::widget::{Element, Widget, WidgetMeta};

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

impl<Window> Widget<Window> for Fill {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Default::default()
    }

    fn should_update(&self, next_widget: &Self, _next_children: &[Element<Window>]) -> bool {
        self == next_widget
    }

    fn paint(&self, rectangle: &Rectangle, handle: &Window, paint_context: &mut PaintContext<Window>) {
        paint_context.fill_rectangle(self.color, rectangle);
        paint_context.commit(handle, rectangle);
    }
}

impl WidgetMeta for Fill {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
