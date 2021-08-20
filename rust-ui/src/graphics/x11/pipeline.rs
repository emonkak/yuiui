use crate::base::PhysicalRectangle;
use crate::graphics::color::Color;

#[derive(Clone, Debug)]
pub struct Pipeline {
    pub(crate) draw_ops: Vec<DrawOp>,
}

#[derive(Clone, Debug)]
pub enum DrawOp {
    FillRectangle(Color, PhysicalRectangle),
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            draw_ops: Vec::new(),
        }
    }

    pub fn push(&mut self, draw_op: DrawOp) {
        self.draw_ops.push(draw_op);
    }
}
