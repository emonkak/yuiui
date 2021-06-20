use geometrics::Rectangle;
use widget::Widget;

pub struct NullWidget;

impl<WindowHandle: Clone, PaintContext> Widget<WindowHandle, PaintContext> for NullWidget {
    fn connect(&mut self, parent_handle: &WindowHandle, _rectangle: &Rectangle, _paint_context: &mut PaintContext) -> WindowHandle {
        parent_handle.clone()
    }
}
