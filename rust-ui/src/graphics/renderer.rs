use super::color::Color;
use super::pipeline::Pipeline;
use super::viewport::Viewport;

pub trait Renderer {
    type Frame;

    type Pipeline: self::Pipeline;

    fn create_frame(&mut self, viewport: &Viewport) -> Self::Frame;

    fn create_pipeline(&mut self, viewport: &Viewport) -> Self::Pipeline;

    fn perform_pipeline(
        &mut self,
        frame: &mut Self::Frame,
        pipeline: &mut Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    );
}
