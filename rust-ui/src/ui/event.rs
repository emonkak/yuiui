use crate::event::MouseEvent;
use crate::geometrics::{PhysicalRectangle, PhysicalSize};

#[derive(Debug)]
pub enum Event<Message, WindowId> {
    WindowEvent(WindowId, WindowEvent),
    Message(Message),
}

#[derive(Debug)]
pub enum WindowEvent {
    PointerPressed(MouseEvent),
    PointerReleased(MouseEvent),
    SizeChanged(PhysicalSize),
    Closed,
    RedrawRequested(PhysicalRectangle),
}
