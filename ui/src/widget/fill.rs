use std::any;

use geometrics::Rectangle;
use paint::PaintContext;

use super::{Element, Widget, WidgetMeta};

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

impl<Handle> Widget<Handle> for Fill {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Default::default()
    }

    fn should_update(&self, next_widget: &Self, _next_children: &[Element<Handle>]) -> bool {
        self != next_widget
    }

    fn paint(&self, rectangle: &Rectangle, _handle: &Handle, paint_context: &mut PaintContext<Handle>) {
        paint_context.fill_rectangle(self.color, rectangle);
    }
}

impl WidgetMeta for Fill {
    fn as_any(&self) -> &dyn any::Any {
        self
    }
}
