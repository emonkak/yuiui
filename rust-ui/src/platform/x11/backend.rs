use std::mem;
use std::sync::atomic::{AtomicPtr, Ordering};

use x11::xlib;

use crate::event::mouse::MouseDown;
use crate::event::GenericEvent;
use crate::geometrics::WindowSize;
use crate::platform::backend::{Backend, Message};
use crate::platform::paint::GeneralPainter;

use super::event::XEvent;
use super::paint::XPainter;

const UPDATE_ATOM_NAME: &str = "__RUST_UI_UPDATE\0";

pub struct XBackend {
    display: *mut xlib::Display,
    window: xlib::Window,
    update_atom: xlib::Atom,
}

impl XBackend {
    pub fn new(display: *mut xlib::Display, window: xlib::Window) -> Self {
        unsafe {
            let update_atom =
                xlib::XInternAtom(display, UPDATE_ATOM_NAME.as_ptr() as *const _, xlib::False);
            Self {
                display,
                window,
                update_atom,
            }
        }
    }
}

impl Backend<XPainter> for XBackend {
    fn create_painter(&mut self, window_size: WindowSize) -> XPainter {
        XPainter::new(self.display, self.window, window_size)
    }

    fn commit_paint(&mut self, painter: &mut XPainter) {
        painter.commit()
    }

    fn advance_event_loop(&mut self) -> Message {
        let mut event: xlib::XEvent = unsafe { mem::MaybeUninit::uninit().assume_init() };

        loop {
            unsafe {
                xlib::XNextEvent(self.display, &mut event);
            }

            match XEvent::from(&event) {
                XEvent::Expose(event) if event.window == self.window => {
                    return Message::Invalidate;
                }
                XEvent::ButtonRelease(event) if event.window == self.window => {
                    return Message::Event(GenericEvent::new::<MouseDown>((&event).into()));
                }
                XEvent::ConfigureNotify(event) if event.window == self.window => {
                    return Message::Resize(WindowSize {
                        width: event.width as _,
                        height: event.height as _,
                    });
                }
                XEvent::ClientMessage(event)
                    if event.window == self.window && event.message_type == self.update_atom =>
                {
                    return Message::Update;
                }
                _ => (),
            }
        }
    }

    fn create_notifier(&mut self) -> Box<dyn Fn() + Send> {
        let display = AtomicPtr::new(self.display);
        let window = self.window;
        let update_atom = self.update_atom;
        Box::new(move || unsafe {
            let display = display.load(Ordering::Relaxed);
            let mut event = xlib::XEvent::from(xlib::XClientMessageEvent {
                type_: xlib::ClientMessage,
                serial: 0,
                send_event: xlib::True,
                display,
                window: window,
                message_type: update_atom,
                format: 32,
                data: xlib::ClientMessageData::new(),
            });

            xlib::XSendEvent(display, window, xlib::True, xlib::NoEventMask, &mut event);
            xlib::XFlush(display);
        })
    }
}
