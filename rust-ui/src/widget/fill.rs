use rust_ui_derive::WidgetMeta;

use crate::geometrics::Rectangle;
use crate::graphics::{Background, Color, Primitive};
use crate::paint::PaintContext;

use super::element::Children;
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
    type Message = ();
    type PaintObject = ();

    fn should_render(
        &self,
        _children: &Children<Renderer>,
        _state: &Self::State,
        new_widget: &Self,
        _new_children: &Children<Renderer>,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        _children: &Children<Renderer>,
        _paint_object: &mut Self::PaintObject,
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
