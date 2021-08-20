use std::collections::VecDeque;

use crate::base::PhysicalRectangle;
use crate::graphics::color::Color;
use crate::graphics::draw_pipeline::DrawPipeline as DrawPipelineTrait;

#[derive(Clone, Debug)]
pub struct DrawPipeline {
    pub(crate) draw_ops: VecDeque<DrawOp>,
}

#[derive(Clone, Debug)]
pub enum DrawOp {
    FillRectangle(Color, PhysicalRectangle),
}

impl DrawPipeline {
    pub fn push(&mut self, draw_op: DrawOp) {
        self.draw_ops.push_front(draw_op);
    }

    pub fn batch(&mut self, draw_ops: Vec<DrawOp>) {
        for draw_op in draw_ops.into_iter().rev() {
            self.draw_ops.push_front(draw_op);
        }
    }
}

impl Default for DrawPipeline {
    fn default() -> Self {
        DrawPipeline {
            draw_ops: VecDeque::new(),
        }
    }
}

impl DrawPipelineTrait for DrawPipeline {
    fn compose(&mut self, other: Self) {
        if !other.draw_ops.is_empty() {
            self.draw_ops.extend(other.draw_ops)
        }
    }
}
