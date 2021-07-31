use rust_ui_derive::WidgetMeta;

use crate::geometrics::Rectangle;
use crate::paint::{PaintContext, Painter};

use super::element::Children;
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

    fn should_update(
        &self,
        new_widget: &Self,
        _old_children: &Children<Handle>,
        _new_children: &Children<Handle>,
        _state: &Self::State,
    ) -> bool {
        self != new_widget
    }

    fn paint(
        &self,
        rectangle: &Rectangle,
        _state: &mut Self::State,
        context: &mut PaintContext<Handle>,
    ) {
        context.fill_rectangle(self.color, rectangle);
    }
}
