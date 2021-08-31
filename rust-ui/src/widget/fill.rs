use std::any::Any;

use crate::geometrics::Rectangle;
use crate::graphics::{Background, Color, Primitive};

use super::message::MessageEmitter;
use super::widget::{AsAny, ShouldRender, Widget};

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

    fn initial_state(&self) -> Self::State {
        Self::State::default()
    }

    fn draw(
        &self,
        _state: &mut Self::State,
        bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter,
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

impl ShouldRender<Self> for Fill {
    fn should_render(&self, other: &Self) -> bool {
        self != other
    }
}

impl AsAny for Fill {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
