use raw_window_handle::unix::XlibHandle;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::mem::MaybeUninit;
use x11::xlib;

use crate::geometrics::PhysicalRectangle;
use crate::ui::window::Window;

pub struct XWindow {
    pub display: *mut xlib::Display,
    pub window: xlib::Window,
}

impl XWindow {
    pub fn new(display: *mut xlib::Display, width: u32, height: u32) -> Self {
        let window = unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let root = xlib::XRootWindowOfScreen(screen);

            let mut attributes: xlib::XSetWindowAttributes = MaybeUninit::zeroed().assume_init();
            attributes.background_pixel = xlib::XWhitePixel(display, screen_number);

            xlib::XCreateWindow(
                display,
                root,
                0,
                0,
                width,
                height,
                0,
                xlib::CopyFromParent,
                xlib::InputOutput as u32,
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CWBackPixel,
                &mut attributes,
            )
        };

        Self { display, window }
    }
}

impl Window for XWindow {
    type WindowId = xlib::Window;

    fn window_id(&self) -> Self::WindowId {
        self.window
    }

    fn get_bounds(&self) -> PhysicalRectangle {
        unsafe {
            let mut attributes_ptr: MaybeUninit<xlib::XWindowAttributes> = MaybeUninit::uninit();
            xlib::XGetWindowAttributes(self.display, self.window, attributes_ptr.as_mut_ptr());
            let attributes = attributes_ptr.assume_init();
            PhysicalRectangle {
                x: attributes.x as _,
                y: attributes.y as _,
                width: attributes.width as _,
                height: attributes.height as _,
            }
        }
    }

    fn invalidate(&self, bounds: PhysicalRectangle) {
        let mut event = xlib::XEvent::from(xlib::XExposeEvent {
            type_: xlib::Expose,
            serial: 0,
            send_event: xlib::True,
            display: self.display,
            window: self.window,
            x: bounds.x as _,
            y: bounds.y as _,
            width: bounds.width as _,
            height: bounds.height as _,
            count: 0,
        });

        unsafe {
            xlib::XSendEvent(
                self.display,
                self.window,
                xlib::True,
                xlib::NoEventMask,
                &mut event,
            );
            xlib::XFlush(self.display);
        }
    }
}

unsafe impl HasRawWindowHandle for XWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = XlibHandle::empty();
        handle.window = self.window;
        handle.display = self.display as *mut _;
        RawWindowHandle::Xlib(handle)
    }
}
