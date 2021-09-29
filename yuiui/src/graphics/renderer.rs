use super::{Color, Primitive};
use crate::geometrics::{Rectangle, Viewport};

pub trait Renderer {
    type Surface;

    type Pipeline;

    fn create_surface(&mut self, viewport: &Viewport) -> Self::Surface;

    fn configure_surface(&mut self, surface: &mut Self::Surface, viewport: &Viewport);

    fn create_pipeline(&mut self, primitive: Primitive) -> Self::Pipeline;

    fn perform_pipeline(
        &mut self,
        pipeline: &mut Self::Pipeline,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        effective_bounds: Option<Rectangle>,
        background_color: Color,
    );
}
