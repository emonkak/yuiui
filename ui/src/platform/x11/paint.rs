use std::ptr;
use x11::xlib;

use platform::WindowHandle;
use geometrics::Rectangle;
use paint::Painter;
use super::window::XWindowHandle;

pub struct XPainter {
    display: *mut xlib::Display,
    pixmap: xlib::Pixmap,
    gc: xlib::GC,
}

impl XPainter {
    pub fn new(handle: &XWindowHandle) -> Self {
        let rectangle = handle.get_window_rectangle();
        unsafe {
            let pixmap = {
                let screen = xlib::XDefaultScreenOfDisplay(handle.display);
                let screen_number = xlib::XScreenNumberOfScreen(screen);
                let depth = xlib::XDefaultDepth(handle.display, screen_number);
                xlib::XCreatePixmap(
                    handle.display,
                    handle.window,
                    rectangle.size.width as _,
                    rectangle.size.height as _,
                    depth as _
                )
            };
            let gc = xlib::XCreateGC(handle.display, pixmap, 0, ptr::null_mut());

            {
                let screen = xlib::XDefaultScreenOfDisplay(handle.display);
                let screen_number = xlib::XScreenNumberOfScreen(screen);
                let color = xlib::XWhitePixel(handle.display, screen_number);

                xlib::XSetForeground(handle.display, gc, color);
                xlib::XFillRectangle(
                    handle.display,
                    pixmap,
                    gc,
                    0,
                    0,
                    rectangle.size.width as _,
                    rectangle.size.height as _
                );
            }

            Self {
                display: handle.display,
                pixmap,
                gc
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

impl Painter<XWindowHandle> for XPainter  {
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
                rectangle.size.height as _
            );
        }
    }

    fn commit(&mut self, handle: &XWindowHandle, rectangle: &Rectangle) {
        unsafe {
            xlib::XCopyArea(
                self.display,
                self.pixmap,
                handle.window,
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
