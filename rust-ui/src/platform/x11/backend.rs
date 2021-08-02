use std::mem;
use std::sync::atomic::{AtomicPtr, Ordering};
use x11::xlib;

use crate::event::mouse::MouseDown;
use crate::event::EventType;
use crate::geometrics::WindowSize;
use crate::platform::backend::{Backend, Message};
use crate::platform::paint::GeneralPainter;
use crate::tree::NodeId;

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
        let update_atom = unsafe {
            xlib::XInternAtom(display, UPDATE_ATOM_NAME.as_ptr() as *const _, xlib::False)
        };
        Self {
            display,
            window,
            update_atom,
        }
    }
}

impl Backend<XPainter> for XBackend {
    fn begin_paint(&mut self, window_size: WindowSize) -> XPainter {
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
                XEvent::ClientMessage(event) if event.message_type == self.update_atom => {
                    let node_id = event.data.get_long(0) as _;
                    return Message::Update(node_id);
                }
                _ => (),
            }
        }
    }

    fn create_notifier(&mut self) -> Box<dyn Fn(NodeId) + Send> {
        let display = AtomicPtr::new(self.display);
        let window = self.window;
        let update_atom = self.update_atom;
        Box::new(move |node_id| {
            let display = display.load(Ordering::Relaxed);

            let mut data = xlib::ClientMessageData::new();
            data.set_long(0, node_id as _);

            let mut event = xlib::XEvent::from(xlib::XClientMessageEvent {
                type_: xlib::ClientMessage,
                serial: 0,
                send_event: xlib::True,
                display,
                window: window,
                message_type: update_atom,
                format: 32,
                data,
            });

            unsafe {
                xlib::XSendEvent(display, window, xlib::True, xlib::NoEventMask, &mut event);
                xlib::XFlush(display);
            }
        })
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
