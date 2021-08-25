use crate::graphics::Primitive;

use super::color::Color;
use super::viewport::Viewport;

pub trait Renderer {
    type Surface;

    type Pipeline;

    fn create_surface(&mut self, viewport: &Viewport) -> Self::Surface;

    fn configure_surface(&mut self, surface: &mut Self::Surface, viewport: &Viewport);

    fn create_pipeline(&mut self, viewport: &Viewport) -> Self::Pipeline;

    fn perform_pipeline(
        &mut self,
        surface: &mut Self::Surface,
        pipeline: &mut Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    );

    fn update_pipeline(
        &mut self,
        pipeline: &mut Self::Pipeline,
        primitive: &Primitive,
        depth: usize,
    );

    fn finish_pipeline(&mut self, _pipeline: &mut Self::Pipeline) {}
}
