pub mod x11;

use crate::event::GenericEvent;
use crate::geometrics::Rectangle;

pub enum Message {
    Invalidate,
    Resize((u32, u32)),
    Update,
    Event(GenericEvent),
    Quit,
}

pub trait GeneralPainter {
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, rectangle: &Rectangle);
}

pub trait Backend<Painter> {
    fn initialize(&mut self);

    fn create_painter(&mut self) -> Painter;

    fn create_notifier(&mut self) -> Box<dyn Fn() + Send>;

    fn commit_paint(&mut self, painter: &mut Painter, rectangle: &Rectangle);

    fn advance_event_loop(&mut self) -> Message;

    fn get_window_size(&self) -> (u32, u32);
}
