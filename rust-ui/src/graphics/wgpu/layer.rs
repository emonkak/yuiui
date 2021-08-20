use crate::base::{Point, Rectangle, Vector};

use crate::graphics::background::Background;
use crate::graphics::viewport::Viewport;

use super::draw_op::DrawOp;

#[derive(Debug, Clone)]
pub struct Layer {
    pub bounds: Rectangle,
    pub quads: Vec<Quad>,
}

impl Layer {
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            quads: Vec::new(),
        }
    }

    pub fn generate(draw_op: &DrawOp, viewport: &Viewport) -> Vec<Self> {
        let first_layer = Layer::new(Rectangle::new(Point::ZERO, viewport.logical_size()));

        let mut layers = vec![first_layer];

        Self::process_draw_op(&mut layers, Vector { x: 0.0, y: 0.0 }, draw_op, 0);

        layers
    }

    fn process_draw_op(
        layers: &mut Vec<Self>,
        translation: Vector,
        draw_op: &DrawOp,
        current_layer: usize,
    ) {
        match draw_op {
            DrawOp::None => {}
            DrawOp::Group(draw_ops) => {
                for draw_op in draw_ops {
                    Self::process_draw_op(layers, translation, draw_op, current_layer)
                }
            }
            DrawOp::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                let layer = &mut layers[current_layer];

                layer.quads.push(Quad {
                    position: [bounds.x + translation.x, bounds.y + translation.y],
                    size: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: *border_radius,
                    border_width: *border_width,
                    border_color: border_color.into_linear(),
                });
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Quad {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
}

unsafe impl bytemuck::Zeroable for Quad {}

unsafe impl bytemuck::Pod for Quad {}
