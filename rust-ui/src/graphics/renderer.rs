use super::color::Color;
use super::draw_pipeline::DrawPipeline;
use super::viewport::Viewport;

pub trait Renderer {
    type DrawArea;

    type DrawPipeline: DrawPipeline;

    fn create_draw_area(&mut self, viewport: &Viewport) -> Self::DrawArea;

    fn perform_draw(
        &mut self,
        draw_pipeline: &Self::DrawPipeline,
        draw_area: &mut Self::DrawArea,
        viewport: &Viewport,
        background_color: Color,
    );
}
