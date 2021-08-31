use std::any::Any;

use crate::geometrics::Rectangle;
use crate::graphics::{Background, Color, Primitive};

use super::message::MessageEmitter;
use super::widget::{AsAny, Widget};

#[derive(PartialEq)]
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

    fn should_render(
        &self,
        _state: &Self::State,
        new_widget: &Self,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        _state: &mut Self::State,
        bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter<Self::Message>,
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

impl AsAny for Fill {
    fn as_any(&self) -> &dyn Any {
       self
    }
}
