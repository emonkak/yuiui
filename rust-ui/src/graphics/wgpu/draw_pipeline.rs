use std::collections::VecDeque;

use crate::base::Rectangle;
use crate::graphics::background::Background;
use crate::graphics::color::Color;
use crate::graphics::draw_pipeline::DrawPipeline as DrawPipelineTrait;

use super::quad::Quad;

#[derive(Debug, Clone)]
pub struct DrawPipeline {
    pub(crate) quads: VecDeque<Quad>,
    pub(crate) layers: Vec<DrawLayer>,
}

#[derive(Debug, Clone)]
pub enum DrawOp {
    Group(Vec<DrawOp>),
    Quad {
        bounds: Rectangle,
        background: Background,
        border_radius: f32,
        border_width: f32,
        border_color: Color,
    },
}

#[derive(Debug, Clone)]
pub struct DrawLayer {
    pub(crate) pipeline: DrawPipeline,
    pub(crate) bounds: Rectangle,
}

impl DrawPipeline {
    pub fn push(&mut self, draw_op: DrawOp) {
        match draw_op {
            DrawOp::Group(draw_ops) => {
                for draw_op in draw_ops.into_iter().rev() {
                    self.push(draw_op)
                }
            }
            DrawOp::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                self.quads.push_front(Quad {
                    position: [bounds.x, bounds.y],
                    size: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: border_radius,
                    border_width: border_width,
                    border_color: border_color.into_linear(),
                });
            }
        }
    }
}

impl DrawPipelineTrait for DrawPipeline {
    fn compose(&mut self, other: Self) {
        if !other.layers.is_empty() {
            self.layers.extend(other.layers)
        }
        if !other.quads.is_empty() {
            self.quads.extend(other.quads)
        }
    }
}

impl Default for DrawPipeline {
    fn default() -> Self {
        Self {
            layers: Vec::new(),
            quads: VecDeque::new(),
        }
    }
}
