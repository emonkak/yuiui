use super::color::Color;
use super::viewport::Viewport;

pub trait Renderer {
    type View;

    type Pipeline;

    fn create_view(&mut self, viewport: &Viewport) -> Self::View;

    fn create_pipeline(&mut self, viewport: &Viewport) -> Self::Pipeline;

    fn perform_pipeline(
        &mut self,
        view: &mut Self::View,
        pipeline: &Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    );
}
