use rust_ui_derive::WidgetMeta;

use crate::geometrics::Rectangle;
use crate::painter::PaintContext;

use super::{Widget, WidgetMeta};

#[derive(Eq, PartialEq, WidgetMeta)]
pub struct Fill {
    color: u32,
}

impl Fill {
    pub fn new(color: u32) -> Fill {
        Fill { color }
    }
}

impl<Handle> Widget<Handle> for Fill {
    type State = ();

    fn should_update(&self, new_widget: &Self, _state: &Self::State) -> bool {
        self != new_widget
    }

    fn paint(
        &self,
        rectangle: &Rectangle,
        _state: &mut Self::State,
        paint_context: &mut dyn PaintContext<Handle>,
    ) {
        println!("Fill::paint(): 0x{:08x}", self.color);
        paint_context.fill_rectangle(self.color, rectangle);
    }
}
