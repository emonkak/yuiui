use std::any::Any;

use crate::geometrics::Rectangle;
use crate::graphics::{Color, Primitive};
use crate::text::{FontDescriptor, HorizontalAlign, VerticalAlign};

use super::message::MessageEmitter;
use super::widget::{AsAny, ShouldRender, Widget};

#[derive(PartialEq)]
pub struct Text {
    pub content: String,
    pub color: Color,
    pub font: FontDescriptor,
    pub font_size: f32,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
}

impl<Renderer> Widget<Renderer> for Text {
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
        Primitive::Text {
            bounds,
            content: self.content.clone(),
            color: self.color,
            font: self.font.clone(),
            font_size: self.font_size,
            horizontal_align: self.horizontal_align,
            vertical_align: self.vertical_align,
        }
        .into()
    }
}

impl ShouldRender<Self> for Text {
    fn should_render(&self, other: &Self) -> bool {
        self != other
    }
}

impl AsAny for Text {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
