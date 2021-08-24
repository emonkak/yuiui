use super::color::Color;
use super::pipeline::Pipeline;
use super::viewport::Viewport;

pub trait Renderer {
    type Surface;

    type Pipeline: self::Pipeline;

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
}
