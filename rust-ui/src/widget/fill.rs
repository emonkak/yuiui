use rust_ui_derive::WidgetMeta;
use std::sync::Arc;

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
        _children: &Children<Renderer>,
        _state: StateCell<Self::State>,
        new_widget: &Self,
        _new_children: &Children<Renderer>,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        self: Arc<Self>,
        _children: Children<Renderer>,
        _state: StateCell<Self::State>,
        bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut PaintContext,
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
