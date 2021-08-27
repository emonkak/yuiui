use rust_ui_derive::WidgetMeta;

use crate::geometrics::Rectangle;
use crate::graphics::{Background, Color, Primitive};
use crate::paint::PaintContext;

use super::element::Children;
use super::state::StateCell;
use super::widget::{Widget, WidgetMeta};

#[derive(PartialEq, WidgetMeta)]
pub struct Fill {
    color: Color,
}

impl Fill {
    pub fn new(color: Color) -> Fill {
        Fill { color }
    }
}

impl<Renderer> Widget<Renderer> for Fill {
    type State = ();

    fn should_update(
        &self,
        new_widget: &Self,
        _old_children: &Children<Renderer>,
        _new_children: &Children<Renderer>,
        _state: StateCell<Self::State>,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _state: StateCell<Self::State>,
        _renderer: &mut Renderer,
        _context: &mut PaintContext<Renderer>,
    ) -> Option<Primitive> {
        Primitive::Quad {
            bounds,
            background: Background::Color(self.color),
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
        .into()
    }
}
