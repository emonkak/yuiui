use std::mem;
use x11::xlib;

use geometrics::Size;

#[derive(Debug)]
pub struct XWindowHandle {
    pub display: *mut xlib::Display,
    pub window: xlib::Window
}

impl XWindowHandle {
    pub fn new(display: *mut xlib::Display, window: xlib::Window) -> Self {
        Self {
            display,
            window,
        }
    }

    pub fn show(&self) {
        unsafe {
            xlib::XMapWindow(self.display, self.window);
            xlib::XFlush(self.display);
        }
    }

    pub fn close(&self) {
        unsafe {
            xlib::XDestroyWindow(self.display, self.window);
        }
    }

    pub fn get_size(&self) -> Size {
        let mut attributes: xlib::XWindowAttributes = unsafe { mem::MaybeUninit::zeroed().assume_init() };
        unsafe {
            xlib::XGetWindowAttributes(
                self.display,
                self.window,
                &mut attributes
            );
        }
        Size {
            width: attributes.width as _,
            height: attributes.height as _
        }
    }
}

pub unsafe fn create_window(display: *mut xlib::Display, width: u32, height: u32) -> xlib::Window {
    let screen = xlib::XDefaultScreenOfDisplay(display);
    let screen_number = xlib::XScreenNumberOfScreen(screen);
    let root = xlib::XRootWindowOfScreen(screen);

    let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
    attributes.background_pixel = xlib::XWhitePixel(display, screen_number);

    let window = xlib::XCreateWindow(
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
        &mut attributes
    );

    xlib::XSelectInput(
        display,
        window,
        xlib::ExposureMask
    );

    window
}

