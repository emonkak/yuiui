use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token, Waker};
use std::collections::HashMap;
use std::io;
use std::mem::MaybeUninit;
use std::os::raw::*;
use std::ptr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};
use x11::xlib;

use crate::event::mouse::MouseDown;
use crate::event::window::{WindowClose, WindowCloseEvent, WindowResize};
use crate::event::EventType;
use crate::geometrics::{PhysicalPoint, PhysicalSize};
use crate::ui::event_loop::{ControlFlow, Event, EventLoop, EventLoopProxy, StartCause};

const WINDOE_EVENT_TOKEN: Token = Token(0);
const REDRAW_REQUEST_TOKEN: Token = Token(1);

pub struct XEventLoop {
    display: *mut xlib::Display,
    poll: Poll,
    redraw_receiver: Receiver<xlib::Window>,
    redraw_sender: Sender<xlib::Window>,
    waker: Arc<Waker>,
    window_states: HashMap<xlib::Window, XWindowState>,
}

pub struct XEventLoopProxy {
    waker: Arc<Waker>,
    redraw_sender: Sender<xlib::Window>,
}

#[derive(Default)]
struct XWindowState {
    point: PhysicalPoint,
    size: PhysicalSize,
}

impl XEventLoop {
    pub fn new(display: *mut xlib::Display) -> io::Result<Self> {
        let poll = Poll::new()?;

        let fd = unsafe { xlib::XConnectionNumber(display) };
        poll.registry()
            .register(&mut SourceFd(&fd), WINDOE_EVENT_TOKEN, Interest::READABLE)?;

        let (redraw_sender, redraw_receiver) = channel();
        let waker = Arc::new(Waker::new(poll.registry(), REDRAW_REQUEST_TOKEN)?);

        Ok(Self {
            display,
            poll,
            redraw_receiver,
            redraw_sender,
            waker,
            window_states: HashMap::new(),
        })
    }
}

impl XEventLoop {
    fn drain_events<F>(&mut self, callback: &mut F, control_flow: &mut ControlFlow)
    where
        F: FnMut(Event<xlib::Window>) -> ControlFlow,
    {
        let mut event_ptr = MaybeUninit::uninit();

        unsafe extern "C" fn predicate(
            _display: *mut xlib::Display,
            _event: *mut xlib::XEvent,
            _arg: *mut c_char,
        ) -> c_int {
            1
        }

        while unsafe {
            xlib::XCheckIfEvent(
                self.display,
                event_ptr.as_mut_ptr(),
                Some(predicate),
                ptr::null_mut(),
            )
        } != 0
        {
            let mut event = unsafe { event_ptr.assume_init() };
            self.process_event(callback, control_flow, &mut event);
        }
    }

    fn process_event<F>(
        &mut self,
        callback: &mut F,
        control_flow: &mut ControlFlow,
        event: &mut xlib::XEvent,
    ) where
        F: FnMut(Event<xlib::Window>) -> ControlFlow,
    {
        match unsafe { event.type_ } {
            xlib::Expose => {
                let event: &xlib::XExposeEvent = event.as_ref();

                // Handle only the last XExposeEvent
                if event.count == 0 {
                    let window_state = self.window_states.entry(event.window).or_default();
                    window_state.point = PhysicalPoint {
                        x: event.x as _,
                        y: event.y as _,
                    };
                    window_state.size = PhysicalSize {
                        width: event.width as _,
                        height: event.height as _,
                    };
                    control_flow.trans(callback(Event::RedrawRequested(event.window)));
                }
            }
            xlib::ButtonRelease => {
                let event: &xlib::XButtonEvent = event.as_ref();
                control_flow.trans(callback(Event::WindowEvent(
                    event.window,
                    MouseDown::of(event),
                )));
            }
            xlib::DestroyNotify => {
                let event: &xlib::XDestroyWindowEvent = event.as_ref();
                control_flow.trans(callback(Event::WindowEvent(
                    event.window,
                    WindowClose::of(WindowCloseEvent),
                )));
            }
            xlib::ConfigureNotify => {
                let event: &xlib::XConfigureEvent = event.as_ref();
                let window_state = self.window_states.entry(event.window).or_default();

                if event.width != window_state.size.width as _
                    || event.height != window_state.size.height as _
                {
                    let size = PhysicalSize {
                        width: event.width as _,
                        height: event.height as _,
                    };
                    window_state.size = size;
                    control_flow.trans(callback(Event::WindowEvent(
                        event.window,
                        WindowResize::of(size),
                    )));
                }
            }
            _ => (),
        }
    }
}

impl EventLoop for XEventLoop {
    type WindowId = xlib::Window;
    type Proxy = XEventLoopProxy;

    fn create_proxy(&self) -> Self::Proxy {
        XEventLoopProxy {
            waker: Arc::clone(&self.waker),
            redraw_sender: self.redraw_sender.clone(),
        }
    }

    fn run<F>(&mut self, mut callback: F)
    where
        F: FnMut(Event<xlib::Window>) -> ControlFlow,
    {
        let mut control_flow = ControlFlow::default();
        let mut events = Events::with_capacity(2);
        let mut cause = StartCause::Init;

        loop {
            let start = Instant::now();
            let timeout;
            let deadline;

            control_flow.trans(callback(Event::Tick(cause)));

            match control_flow {
                ControlFlow::Exit => break,
                ControlFlow::Poll => {
                    cause = StartCause::Poll;
                    deadline = None;
                    timeout = Some(Duration::from_millis(0));
                }
                ControlFlow::Wait => {
                    cause = StartCause::WaitCancelled {
                        start,
                        requested_resume: None,
                    };
                    deadline = None;
                    timeout = None;
                }
                ControlFlow::WaitUntil(wait_deadline) => {
                    cause = StartCause::ResumeTimeReached {
                        start,
                        requested_resume: wait_deadline,
                    };
                    timeout = if wait_deadline > start {
                        Some(wait_deadline - start)
                    } else {
                        Some(Duration::from_millis(0))
                    };
                    deadline = Some(wait_deadline);
                }
            }

            self.poll.poll(&mut events, timeout).unwrap();

            for event in &events {
                match event.token() {
                    WINDOE_EVENT_TOKEN => {
                        self.drain_events(&mut callback, &mut control_flow);
                    }
                    REDRAW_REQUEST_TOKEN => {
                        let window = self.redraw_receiver.recv().unwrap();
                        control_flow.trans(callback(Event::RedrawRequested(window)));
                    }
                    _ => {}
                }
            }

            control_flow.trans(callback(Event::EventsConsumed));

            let wait_cancelled = deadline.map_or(false, |deadline| Instant::now() < deadline);

            if wait_cancelled {
                cause = StartCause::WaitCancelled {
                    start,
                    requested_resume: deadline,
                };
            }
        }

        callback(Event::LoopExited);
    }
}

impl EventLoopProxy for XEventLoopProxy {
    type WindowId = xlib::Window;

    fn request_redraw(&self, window_id: Self::WindowId) {
        self.redraw_sender.send(window_id).unwrap();
        self.waker.wake().unwrap();
    }
}
