use std::ptr;
use x11::xlib;

use crate::geometrics::{Rectangle, WindowSize};
use crate::platform::paint::GeneralPainter;

pub struct XPainter {
    display: *mut xlib::Display,
    window: xlib::Window,
    window_size: WindowSize,
    pixmap: xlib::Pixmap,
    gc: xlib::GC,
}

impl XPainter {
    pub fn new(display: *mut xlib::Display, window: xlib::Window, window_size: WindowSize) -> Self {
        unsafe {
            let pixmap = {
                let screen = xlib::XDefaultScreenOfDisplay(display);
                let screen_number = xlib::XScreenNumberOfScreen(screen);
                let depth = xlib::XDefaultDepth(display, screen_number);
                xlib::XCreatePixmap(
                    display,
                    window,
                    window_size.width,
                    window_size.height,
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
                window_size.width,
                window_size.height,
            );

            Self {
                display,
                window,
                pixmap,
                gc,
                window_size,
            }
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

    fn commit(&mut self) {
        unsafe {
            xlib::XCopyArea(
                self.display,
                self.pixmap,
                self.window,
                self.gc,
                0,
                0,
                self.window_size.width,
                self.window_size.height,
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
