use std::any::Any;

use crate::geometrics::Rectangle;
use crate::graphics::{Color, Primitive};
use crate::text::{FontDescriptor, HorizontalAlign, VerticalAlign};

use super::message::MessageEmitter;
use super::paint_object::PaintObject;
use super::state::StateContainer;
use super::widget::{Widget, WidgetSeal};

#[derive(PartialEq)]
pub struct Text {
    pub content: String,
    pub color: Color,
    pub font: FontDescriptor,
    pub font_size: f32,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
}

pub struct TextPaint;

impl<Renderer: 'static> Widget<Renderer> for Text {
    type State = TextPaint;
    type Message = ();

    fn initial_state(&self) -> StateContainer<Renderer, Self, Self::State, Self::Message> {
        StateContainer::from_paint_object(TextPaint)
    }

    fn should_render(&self, other: &Self) -> bool {
        self != other
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

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WidgetSeal for Text {}

impl<Renderer> PaintObject<Renderer> for TextPaint {
    type Widget = Text;

    type Message = ();

    fn draw(
        &mut self,
        widget: &Self::Widget,
        bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter,
    ) -> Option<Primitive> {
        Primitive::Text {
            bounds,
            content: widget.content.clone(),
            color: widget.color,
            font: widget.font.clone(),
            font_size: widget.font_size,
            horizontal_align: widget.horizontal_align,
            vertical_align: widget.vertical_align,
        }
        .into()
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
