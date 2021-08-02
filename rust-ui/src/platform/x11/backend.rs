use std::ptr;
use std::mem;
use std::sync::atomic::{AtomicPtr, Ordering};

use x11::xlib;

use crate::event::GenericEvent;
use crate::event::mouse::MouseDown;
use crate::geometrics::Rectangle;
use crate::platform::{Backend, GeneralPainter, Message};

use super::event::XEvent;
use super::window;

const UPDATE_ATOM_NAME: &str = "__RUST_UI_UPDATE\0";

pub struct XBackend {
    display: *mut xlib::Display,
    window: xlib::Window,
    update_atom: xlib::Atom,
}

pub struct XPainter {
    display: *mut xlib::Display,
    window: xlib::Window,
    pixmap: xlib::Pixmap,
    gc: xlib::GC,
}

impl XBackend {
    pub fn new(
        display: *mut xlib::Display,
        window: xlib::Window,
    ) -> Self {
        unsafe {
            let update_atom = xlib::XInternAtom(
                display,
                UPDATE_ATOM_NAME.as_ptr() as *const _,
                xlib::False
            );
            Self {
                display,
                window,
                update_atom,
            }
        }
    }
}

impl Backend<XPainter> for XBackend {
    fn initialize(&mut self) {
        unsafe {
            xlib::XMapWindow(self.display, self.window);
            xlib::XFlush(self.display);
        }
    }

    fn create_painter(&mut self) -> XPainter {
        let (_, _, width, height) = unsafe {
            window::get_window_rectangle(self.display, self.window)
        };
        XPainter::new(self.display, self.window, width, height)
    }

    fn commit_paint(&mut self, painter: &mut XPainter, rectangle: &Rectangle) {
        painter.commit(rectangle)
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
                    return Message::Resize((event.width as _, event.height as _));
                }
                XEvent::ClientMessage(event) if event.window == self.window && event.message_type == self.update_atom => {
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
        Box::new(move || {
            unsafe {
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
            }
        })
    }

    fn get_window_size(&self) -> (u32, u32) {
        let (_, _, width, height) = unsafe {
            window::get_window_rectangle(self.display, self.window)
        };
        (width, height)
    }
}

impl XPainter {
    pub fn new(display: *mut xlib::Display, window: xlib::Window, width: u32, height: u32) -> Self {
        unsafe {
            let pixmap = {
                let screen = xlib::XDefaultScreenOfDisplay(display);
                let screen_number = xlib::XScreenNumberOfScreen(screen);
                let depth = xlib::XDefaultDepth(display, screen_number);
                xlib::XCreatePixmap(
                    display,
                    window,
                    width,
                    height,
                    depth as _,
                )
            };
            let gc = xlib::XCreateGC(display, pixmap, 0, ptr::null_mut());

            let screen = xlib::XDefaultScreenOfDisplay(display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let color = xlib::XWhitePixel(display, screen_number);

            xlib::XSetForeground(display, gc, color);
            xlib::XFillRectangle(
                display,
                pixmap,
                gc,
                0,
                0,
                width,
                height,
            );

            Self { display, window, pixmap, gc }
        }
    }

    fn alloc_color(&self, rgba: u32) -> xlib::XColor {
        let mut color = xlib::XColor {
            pixel: 0,
            red: (((rgba & 0xff000000) >> 24) * 0x101) as u16,
            green: (((rgba & 0x00ff0000) >> 16) * 0x101) as u16,
            blue: (((rgba & 0x0000ff00) >> 8) * 0x101) as u16,
            flags: 0,
            pad: 0,
        };

        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(self.display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let colormap = xlib::XDefaultColormap(self.display, screen_number);
            xlib::XAllocColor(self.display, colormap, &mut color);
        };

        color
    }
}

impl GeneralPainter for XPainter {
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle) {
        unsafe {
            let color = self.alloc_color(color);
            xlib::XSetForeground(self.display, self.gc, color.pixel);
            xlib::XFillRectangle(
                self.display,
                self.pixmap,
                self.gc,
                rectangle.point.x as _,
                rectangle.point.y as _,
                rectangle.size.width as _,
                rectangle.size.height as _,
            );
        }
    }

    fn commit(&mut self, rectangle: &Rectangle) {
        unsafe {
            xlib::XCopyArea(
                self.display,
                self.pixmap,
                self.window,
                self.gc,
                0,
                0,
                rectangle.size.width as _,
                rectangle.size.height as _,
                0,
                0,
            );
        }
    }
}

impl Drop for XPainter {
    fn drop(&mut self) {
        unsafe {
            xlib::XFreeGC(self.display, self.gc);
            xlib::XFreePixmap(self.display, self.pixmap);
        }
    }
}
