use super::mouse::MouseEvent;
use crate::geometrics::{PhysicalRectangle, PhysicalSize};

#[derive(Debug)]
pub enum WindowEvent {
    PointerPressed(MouseEvent),
    PointerReleased(MouseEvent),
    SizeChanged(PhysicalSize),
    Closed,
    RedrawRequested(PhysicalRectangle),
}