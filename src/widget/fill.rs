use std::any;

use crate::geometrics::Rectangle;
use crate::paint::PaintContext;

use super::{Widget, WidgetMeta};

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

    fn should_update(&self, new_widget: &Self, _state: &Self::State) -> bool {
        self != new_widget
    }

    fn paint(&self, _handle: &Handle, rectangle: &Rectangle, _state: &mut Self::State, paint_context: &mut dyn PaintContext<Handle>) {
        paint_context.fill_rectangle(self.color, rectangle);
    }
}

impl WidgetMeta for Fill {
    fn as_any(&self) -> &dyn any::Any {
        self
    }
}
