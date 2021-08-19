use std::time::Instant;

use crate::event::GenericEvent;

pub enum ControlFlow {
    Exit,
    Poll,
    Wait,
    WaitUntil(Instant),
}

pub enum StartCause {
    Init,
    Poll,
    WaitCancelled {
        start: Instant,
        requested_resume: Option<Instant>,
    },
    ResumeTimeReached {
        start: Instant,
        requested_resume: Instant,
    },
}

pub enum Event<WindowId> {
    Tick(StartCause),
    WindowEvent(WindowId, GenericEvent),
    RedrawRequested(WindowId),
    EventsConsumed,
    LoopExited,
}

pub trait EventLoop {
    type WindowId;
    type Proxy: EventLoopProxy<WindowId = Self::WindowId>;

    fn create_proxy(&self) -> Self::Proxy;

    fn run<F>(&mut self, callback: F)
    where
        F: FnMut(Event<Self::WindowId>) -> ControlFlow;
}

pub trait EventLoopProxy: Send {
    type WindowId;

    fn request_redraw(&self, window: Self::WindowId);
}

impl ControlFlow {
    pub fn trans(&mut self, next: Self) {
        match self {
            Self::Exit => {}
            _ => {
                *self = next;
            }
        }
    }
}

impl Default for ControlFlow {
    #[inline]
    fn default() -> ControlFlow {
        ControlFlow::Poll
    }
}
