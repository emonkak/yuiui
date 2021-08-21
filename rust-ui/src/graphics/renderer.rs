use super::{Color, Viewport};

pub trait Renderer {
    type Frame;

    type Primitive: Default;

    type Pipeline: self::Pipeline<Self::Primitive>;

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

pub trait Pipeline<Primitive> {
    fn push(&mut self, primitive: &Primitive);
}
