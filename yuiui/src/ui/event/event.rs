use super::mouse::MouseEvent;
use crate::geometrics::{PhysicalRectangle, PhysicalSize};

#[derive(Debug)]
pub enum Event<Message, WindowId> {
    LoopInitialized,
    Message(Message),
    WindowEvent(WindowId, WindowEvent),
}

#[derive(Debug)]
pub enum WindowEvent {
    PointerPressed(MouseEvent),
    PointerReleased(MouseEvent),
    SizeChanged(PhysicalSize),
    Closed,
    RedrawRequested(PhysicalRectangle),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[rustfmt::skip]
pub enum WindowEventMask {
    PointerPressed  = 1 << 2,
    PointerReleased = 1 << 3,
    SizeChanged     = 1 << 4,
    Closed          = 1 << 5,
    RedrawRequested = 1 << 6,
}

impl WindowEvent {
    pub fn event_mask(&self) -> WindowEventMask {
        match self {
            Self::PointerPressed(_) => WindowEventMask::PointerPressed,
            Self::PointerReleased(_) => WindowEventMask::PointerReleased,
            Self::SizeChanged(_) => WindowEventMask::SizeChanged,
            Self::Closed => WindowEventMask::Closed,
            Self::RedrawRequested(_) => WindowEventMask::RedrawRequested,
        }
    }
}

impl Into<usize> for WindowEventMask {
    fn into(self) -> usize {
        self as usize
    }
}
