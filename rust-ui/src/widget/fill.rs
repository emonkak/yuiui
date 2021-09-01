use std::any::Any;

use crate::geometrics::Rectangle;
use crate::graphics::{Background, Color, Primitive};

use super::message::MessageEmitter;
use super::paint_object::PaintObject;
use super::state::StateContainer;
use super::widget::{Widget, WidgetSeal};

#[derive(PartialEq)]
pub struct Fill {
    color: Color,
}

pub struct FillPaint;

impl Fill {
    pub fn new(color: Color) -> Fill {
        Fill { color }
    }
}

impl<Renderer: 'static> Widget<Renderer> for Fill {
    type State = FillPaint;
    type Message = ();

    fn initial_state(&self) -> StateContainer<Renderer, Self, Self::State, Self::Message> {
        StateContainer::from_paint_object(FillPaint)
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
        Primitive::Quad {
            bounds,
            background: Background::Color(self.color),
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
        .into()
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Renderer> PaintObject<Renderer> for FillPaint {
    type Widget = Fill;

    type Message = ();

    fn draw(
        &mut self,
        widget: &Self::Widget,
        bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter,
    ) -> Option<Primitive> {
        Primitive::Quad {
            bounds,
            background: Background::Color(widget.color),
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
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

impl WidgetSeal for Fill {}
