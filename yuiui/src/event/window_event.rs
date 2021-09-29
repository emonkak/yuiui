use super::mouse::MouseEvent;
use crate::geometrics::PhysicalSize;

#[derive(Debug)]
pub enum WindowEvent {
    PointerPressed(MouseEvent),
    PointerReleased(MouseEvent),
    SizeChanged(PhysicalSize),
    Closed,
    RedrawRequested,
}
