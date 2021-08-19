use rust_ui_derive::WidgetMeta;

use crate::base::Rectangle;
use crate::graphics::color::Color;
use crate::graphics::x11::renderer::{DrawOp, XRenderer};
use crate::paint::PaintContext;

use super::element::Children;
use super::{Widget, WidgetMeta};

#[derive(PartialEq, WidgetMeta)]
pub struct Fill {
    color: Color,
}

impl Fill {
    pub fn new(color: Color) -> Fill {
        Fill { color }
    }
}

impl Widget<XRenderer> for Fill {
    type State = ();

    fn should_update(
        &self,
        new_widget: &Self,
        _old_children: &Children<XRenderer>,
        _new_children: &Children<XRenderer>,
        _state: &Self::State,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        draw_op: DrawOp,
        bounds: Rectangle,
        _state: &mut Self::State,
        _renderer: &mut XRenderer,
        _context: &mut PaintContext<XRenderer>,
    ) -> DrawOp {
        DrawOp::FillRectangle(self.color, bounds.into()) + draw_op
    }
}
