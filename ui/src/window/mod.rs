use geometrics::Size;

pub mod x11;

pub trait WindowHandler<WindowHandle, PaintContext> {
    fn connect(&self, handle: &WindowHandle);

    fn size(&self, _width: u32, _height: u32) {
    }

    fn paint(&self, paint_context: &mut PaintContext);

    fn destroy(&self) {
    }
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
