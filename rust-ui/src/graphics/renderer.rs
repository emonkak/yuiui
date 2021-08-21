use super::color::Color;
use super::viewport::Viewport;

pub trait Renderer {
    type DrawArea;

    type Primitive: Default;

    type Pipeline: self::Pipeline<Self::Primitive>;

    fn create_draw_area(&mut self, viewport: &Viewport) -> Self::DrawArea;

    fn create_pipeline(&mut self, viewport: &Viewport) -> Self::Pipeline;

    fn perform_pipeline(
        &mut self,
        draw_area: &mut Self::DrawArea,
        pipeline: &Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    );
}

pub trait Pipeline<Primitive> {
    fn push(&mut self, primitive: &Primitive);
}
