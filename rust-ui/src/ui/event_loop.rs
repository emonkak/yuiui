use std::time::Instant;

use crate::event::WindowEvent;

#[derive(Debug)]
pub enum ControlFlow {
    Exit,
    Poll,
    Wait,
    WaitUntil(Instant),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Event<WindowId> {
    Tick(StartCause),
    WindowEvent(WindowId, WindowEvent),
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
    #[inline]
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
