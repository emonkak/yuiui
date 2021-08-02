use rust_ui_derive::WidgetMeta;

use crate::geometrics::Rectangle;
use crate::paint::{PaintContext, PaintHint};
use crate::platform::paint::GeneralPainter;

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

impl<Painter: GeneralPainter> Widget<Painter> for Fill {
    type State = ();

    fn should_update(
        &self,
        new_widget: &Self,
        _old_children: &Children<Painter>,
        _new_children: &Children<Painter>,
        _state: &Self::State,
    ) -> bool {
        self != new_widget
    }

    fn paint(
        &self,
        rectangle: &Rectangle,
        _state: &mut Self::State,
        painter: &mut Painter,
        _context: &mut PaintContext<Painter>,
    ) -> PaintHint {
        painter.fill_rectangle(self.color, rectangle);
        PaintHint::Always
    }
}
