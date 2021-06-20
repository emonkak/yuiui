use geometrics::Size;
use std::any::Any;

pub mod x11;

pub trait WindowHandler<WindowHandle, PaintContext> {
    fn connect(&self, handle: &WindowHandle);

    fn size(&self, _width: u32, _height: u32) {
    }

    fn paint(&self, paint_context: &mut PaintContext);

    /// Called when the resources need to be rebuilt.
    fn rebuild_resources(&self) {
    }

    fn destroy(&self) {
    }

    fn as_any(&self) -> &dyn Any;
}

pub trait WindowHandle: Clone {
    fn show(&self);

    fn close(&self);

    fn get_size(&self) -> Size;
}

pub trait WindowProcedure<WindowHandle, WindowEvent> {
    fn connect(&self, handle: &WindowHandle);

    fn handle_event(&self, event: &WindowEvent) -> bool;
}
