use std::mem;
use x11::xlib;

use mio::unix::SourceFd;
use mio::{Interest, Poll, Token};

use crate::event::mouse::MouseDown;
use crate::event::EventType;
use crate::geometrics::WindowSize;
use crate::platform::backend::{Backend, Message};
use crate::platform::paint::GeneralPainter;

use super::event::XEvent;
use super::paint::XPainter;

pub struct XBackend {
    display: *mut xlib::Display,
    window: xlib::Window,
}

impl XBackend {
    pub fn new(display: *mut xlib::Display, window: xlib::Window) -> Self {
        Self {
            display,
            window,
        }
    }
}

impl Backend<XPainter> for XBackend {
    fn begin_paint(&mut self, window_size: WindowSize) -> XPainter {
        XPainter::new(self.display, self.window, window_size)
    }

    fn commit_paint(&mut self, painter: &mut XPainter) {
        painter.commit();
        unsafe {
            xlib::XFlush(self.display);
        }
    }

    fn invalidate(&self) {
        let mut event = xlib::XEvent::from(xlib::XExposeEvent {
            type_: xlib::Expose,
            serial: 0,
            send_event: xlib::True,
            display: self.display,
            window: self.window,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            count: 0,
        });

        unsafe {
            xlib::XSendEvent(self.display, self.window, xlib::True, xlib::NoEventMask, &mut event);
            xlib::XFlush(self.display);
        }
    }

    fn advance_event_loop(&mut self) -> Message {
        let mut event: xlib::XEvent = unsafe { mem::MaybeUninit::uninit().assume_init() };

        loop {
            unsafe {
                xlib::XNextEvent(self.display, &mut event);
            }

            if unsafe { event.any.window } != self.window {
                continue;
            }

            match XEvent::from(&event) {
                XEvent::Expose(_) => {
                    return Message::Invalidate;
                }
                XEvent::ButtonRelease(event) => {
                    return Message::Event(MouseDown::of(event));
                }
                XEvent::DestroyNotify(_) => {
                    return Message::Quit;
                }
                XEvent::ConfigureNotify(event) => {
                    return Message::Resize(WindowSize {
                        width: event.width as _,
                        height: event.height as _,
                    });
                }
                _ => (),
            }
        }
    }

    fn subscribe_window_events(&self, poll: &Poll, token: Token) {
        let fd = unsafe { xlib::XConnectionNumber(self.display) };

        poll.registry()
            .register(&mut SourceFd(&fd), token, Interest::READABLE)
            .unwrap();

        unsafe {
            xlib::XFlush(self.display);
        }
    }

    fn get_window_size(&self) -> WindowSize {
        unsafe {
            let mut attributes: xlib::XWindowAttributes = mem::MaybeUninit::zeroed().assume_init();
            xlib::XGetWindowAttributes(self.display, self.window, &mut attributes);
            WindowSize {
                width: attributes.width as _,
                height: attributes.height as _,
            }
        }
    }
}
