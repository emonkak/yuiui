use std::ops::Add;

use crate::base::PhysicalSize;
use crate::graphics::color::Color;

pub trait Renderer {
    type DrawArea;

    type DrawOp: Default + Add<Output = Self::DrawOp>;

    fn create_draw_area(&mut self, size: PhysicalSize) -> Self::DrawArea;

    fn perform_draw(
        &mut self,
        draw_area: &Self::DrawArea,
        draw_op: &Self::DrawOp,
        background_color: Color,
    );
}
