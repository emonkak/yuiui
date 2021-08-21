use std::ptr;
use x11::xlib;

use crate::geometrics::{PhysicalRectangle, PhysicalSize};
use crate::graphics::{Color, Renderer as RendererTrait, Viewport};

use super::pipeline::Pipeline;
use super::primitive::Primitive;

#[derive(Debug)]
pub struct Renderer {
    display: *mut xlib::Display,
    window: xlib::Window,
}

#[derive(Debug)]
pub struct Frame {
    display: *mut xlib::Display,
    pixmap: xlib::Pixmap,
    gc: xlib::GC,
}

impl Renderer {
    pub fn new(display: *mut xlib::Display, window: xlib::Window) -> Self {
        Self { display, window }
    }

    fn alloc_color(&self, color: &Color) -> xlib::XColor {
        let [red, green, blue, _] = color.into_u16_components();

        let mut color = xlib::XColor {
            pixel: 0,
            red,
            green,
            blue,
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

    fn fill_rectangle(&self, frame: &Frame, color: &xlib::XColor, bounds: PhysicalRectangle) {
        unsafe {
            xlib::XSetForeground(self.display, frame.gc, color.pixel);
            xlib::XFillRectangle(
                self.display,
                frame.pixmap,
                frame.gc,
                bounds.x as _,
                bounds.y as _,
                bounds.width as _,
                bounds.height as _,
            );
        }
    }

    fn commit(&self, frame: &Frame, size: PhysicalSize) {
        unsafe {
            xlib::XCopyArea(
                self.display,
                frame.pixmap,
                self.window,
                frame.gc,
                0,
                0,
                size.width,
                size.height,
                0,
                0,
            );
            xlib::XFlush(self.display);
        }
    }

    fn process_primitives(&self, frame: &Frame, primitives: &[Primitive]) {
        for primitive in primitives {
            match primitive {
                Primitive::None => {}
                Primitive::Batch(primitives) => {
                    self.process_primitives(frame, primitives);
                }
                Primitive::FillRectangle(color, bounds) => {
                    let alloc_color = self.alloc_color(color);
                    self.fill_rectangle(frame, &alloc_color, *bounds);
                }
            }
        }
    }
}

impl RendererTrait for Renderer {
    type Frame = self::Frame;
    type Primitive = self::Primitive;
    type Pipeline = self::Pipeline;

    fn create_frame(&mut self, viewport: &Viewport) -> Self::Frame {
        Frame::new(self.display, viewport.physical_size())
    }

    fn create_pipeline(&mut self, _viewport: &Viewport) -> Self::Pipeline {
        Pipeline::new()
    }

    fn perform_pipeline(
        &mut self,
        frame: &mut Self::Frame,
        pipeline: &mut Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    ) {
        let alloc_background_color = self.alloc_color(&background_color);

        self.fill_rectangle(
            frame,
            &alloc_background_color,
            PhysicalRectangle::from(viewport.physical_size()),
        );

        self.process_primitives(frame, &pipeline.primitives());

        self.commit(frame, viewport.physical_size());
    }
}

impl Frame {
    pub fn new(display: *mut xlib::Display, size: PhysicalSize) -> Self {
        unsafe {
            let pixmap = {
                let root = xlib::XDefaultRootWindow(display);
                let screen = xlib::XDefaultScreenOfDisplay(display);
                let screen_number = xlib::XScreenNumberOfScreen(screen);
                let depth = xlib::XDefaultDepth(display, screen_number);
                xlib::XCreatePixmap(display, root, size.width, size.height, depth as _)
            };
            let gc = xlib::XCreateGC(display, pixmap, 0, ptr::null_mut());

            Self {
                display,
                pixmap,
                gc,
            }
        }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            xlib::XFreeGC(self.display, self.gc);
            xlib::XFreePixmap(self.display, self.pixmap);
        }
    }
}
