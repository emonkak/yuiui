use crate::event::WindowEvent;

#[derive(Debug)]
pub enum Event<'a, State> {
    StateChanged(&'a State),
    WindowEvent(WindowEvent),
}

impl<'a, State> From<WindowEvent> for Event<'a, State> {
    fn from(window_event: WindowEvent) -> Self {
        Self::WindowEvent(window_event)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
#[rustfmt::skip]
pub enum EventMask {
    None            = 0,
    StateChanged    = 1 << 0,
    PointerPressed  = 1 << 1,
    PointerReleased = 1 << 2,
    SizeChanged     = 1 << 3,
    Closed          = 1 << 4,
    RedrawRequested = 1 << 5,
}

impl<'a, State> Event<'a, State> {
    pub fn event_mask(&self) -> EventMask {
        match self {
            Self::StateChanged(_) => EventMask::StateChanged,
            Self::WindowEvent(WindowEvent::PointerPressed(_)) => EventMask::PointerPressed,
            Self::WindowEvent(WindowEvent::PointerReleased(_)) => EventMask::PointerReleased,
            Self::WindowEvent(WindowEvent::SizeChanged(_)) => EventMask::SizeChanged,
            Self::WindowEvent(WindowEvent::Closed) => EventMask::Closed,
            Self::WindowEvent(WindowEvent::RedrawRequested(_)) => EventMask::RedrawRequested,
        }
    }
}

impl Into<usize> for EventMask {
    fn into(self) -> usize {
        self as usize
    }
}
