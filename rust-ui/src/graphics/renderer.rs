use std::ops::Add;

use super::color::Color;
use super::viewport::Viewport;

pub trait Renderer {
    type DrawArea;

    type DrawOp: Default + Add<Output = Self::DrawOp>;

    fn create_draw_area(&mut self, viewport: &Viewport) -> Self::DrawArea;

    fn perform_draw(
        &mut self,
        draw_op: &Self::DrawOp,
        draw_area: &mut Self::DrawArea,
        viewport: &Viewport,
        background_color: Color,
    );
}
