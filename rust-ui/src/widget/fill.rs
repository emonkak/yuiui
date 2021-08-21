use rust_ui_derive::WidgetMeta;

use crate::geometrics::Rectangle;
use crate::graphics::{wgpu, x11, Background, Color};
use crate::paint::context::PaintContext;

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

impl Widget<x11::Renderer> for Fill {
    type State = ();

    fn should_update(
        &self,
        new_widget: &Self,
        _old_children: &Children<x11::Renderer>,
        _new_children: &Children<x11::Renderer>,
        _state: &Self::State,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _state: &mut Self::State,
        _renderer: &mut x11::Renderer,
        _context: &mut PaintContext<x11::Renderer>,
    ) -> x11::Primitive {
        x11::Primitive::FillRectangle(self.color, bounds.into())
    }
}

impl Widget<wgpu::Renderer> for Fill {
    type State = ();

    fn should_update(
        &self,
        new_widget: &Self,
        _old_children: &Children<wgpu::Renderer>,
        _new_children: &Children<wgpu::Renderer>,
        _state: &Self::State,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _state: &mut Self::State,
        _renderer: &mut wgpu::Renderer,
        _context: &mut PaintContext<wgpu::Renderer>,
    ) -> wgpu::Primitive {
        wgpu::Primitive::Quad {
            bounds,
            background: Background::Color(self.color),
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}
