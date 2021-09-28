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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
#[rustfmt::skip]
pub enum WindowEventMask {
    None            = 0,
    PointerPressed  = 1 << 0,
    PointerReleased = 1 << 1,
    SizeChanged     = 1 << 2,
    Closed          = 1 << 3,
    RedrawRequested = 1 << 4,
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
