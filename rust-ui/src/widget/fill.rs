use rust_ui_derive::WidgetMeta;

use crate::base::Rectangle;
use crate::graphics::color::Color;
use crate::graphics::background::Background;
use crate::graphics::x11;
use crate::graphics::wgpu;
use crate::paint::LifecycleContext;

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
        draw_op: x11::DrawOp,
        bounds: Rectangle,
        _state: &mut Self::State,
        _renderer: &mut x11::Renderer,
        _context: &mut LifecycleContext<x11::Renderer>,
    ) -> x11::DrawOp {
        x11::DrawOp::FillRectangle(self.color, bounds.into()) + draw_op
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
        draw_op: wgpu::DrawOp,
        bounds: Rectangle,
        _state: &mut Self::State,
        _renderer: &mut wgpu::Renderer,
        _context: &mut LifecycleContext<wgpu::Renderer>,
    ) -> wgpu::DrawOp {
        wgpu::DrawOp::Quad {
            bounds,
            background: Background::Color(self.color),
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        } + draw_op
    }
}
