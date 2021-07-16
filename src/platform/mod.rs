pub mod x11;

use geometrics::Rectangle;

pub trait WindowHandle {
    fn get_window_rectangle(&self) -> Rectangle;

    fn show_window(&self);

    fn close_window(&self);
}
