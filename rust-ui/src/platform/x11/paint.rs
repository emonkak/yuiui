use std::ptr;
use x11::xlib;

use crate::geometrics::Rectangle;
use crate::painter::PaintContext;
use crate::platform::WindowHandle;

use super::window::XWindowHandle;

pub struct XPaintContext<'a> {
    handle: &'a XWindowHandle,
    pixmap: xlib::Pixmap,
    gc: xlib::GC,
}

impl<'a> XPaintContext<'a> {
    pub fn new(handle: &'a XWindowHandle) -> Self {
        let display = handle.display();
        let window = handle.window();
        let rectangle = handle.get_window_rectangle();

        unsafe {
            let pixmap = {
                let screen = xlib::XDefaultScreenOfDisplay(display);
                let screen_number = xlib::XScreenNumberOfScreen(screen);
                let depth = xlib::XDefaultDepth(display, screen_number);
                xlib::XCreatePixmap(
                    display,
                    window,
                    rectangle.size.width as _,
                    rectangle.size.height as _,
                    depth as _,
                )
            };
            let gc = xlib::XCreateGC(display, pixmap, 0, ptr::null_mut());

            {
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
                    rectangle.size.width as _,
                    rectangle.size.height as _,
                );
            }

            Self { handle, pixmap, gc }
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

        let display = self.handle.display();

        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let colormap = xlib::XDefaultColormap(display, screen_number);
            xlib::XAllocColor(display, colormap, &mut color);
        };

        color
    }
}

impl<'a> PaintContext<XWindowHandle> for XPaintContext<'a> {
    fn handle(&self) -> &XWindowHandle {
        self.handle
    }

    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle) {
        let display = self.handle.display();

        unsafe {
            let color = self.alloc_color(color);
            xlib::XSetForeground(display, self.gc, color.pixel);
            xlib::XFillRectangle(
                display,
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
        let display = self.handle.display();
        let window = self.handle.window();

        unsafe {
            xlib::XCopyArea(
                display,
                self.pixmap,
                window,
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

impl<'a> Drop for XPaintContext<'a> {
    fn drop(&mut self) {
        let display = self.handle.display();
        unsafe {
            xlib::XFreeGC(display, self.gc);
            xlib::XFreePixmap(display, self.pixmap);
        }
    }
}
