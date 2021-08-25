use std::ptr;
use x11::xlib;

use crate::geometrics::{PhysicalRectangle, PhysicalSize};
use crate::graphics::{Color, Primitive, Viewport};

use super::pipeline::{DrawOp, Pipeline};

#[derive(Debug)]
pub struct Renderer {
    display: *mut xlib::Display,
    window: xlib::Window,
}

#[derive(Debug)]
pub struct Surface {
    display: *mut xlib::Display,
    pixmap: xlib::Pixmap,
    gc: xlib::GC,
}

impl Renderer {
    pub fn new(display: *mut xlib::Display, window: xlib::Window) -> Self {
        Self { display, window }
    }

    fn fill_rectangle(&self, surface: &Surface, color: &xlib::XColor, bounds: PhysicalRectangle) {
        unsafe {
            xlib::XSetForeground(self.display, surface.gc, color.pixel);
            xlib::XFillRectangle(
                self.display,
                surface.pixmap,
                surface.gc,
                bounds.x as _,
                bounds.y as _,
                bounds.width as _,
                bounds.height as _,
            );
        }
    }

    fn commit(&self, surface: &Surface, size: PhysicalSize) {
        unsafe {
            xlib::XCopyArea(
                self.display,
                surface.pixmap,
                self.window,
                surface.gc,
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

    fn process_draw_op(&self, draw_op: &DrawOp, surface: &Surface) {
        match draw_op {
            DrawOp::FillRectangle(color, bounds) => {
                self.fill_rectangle(surface, color, *bounds);
            }
        }
    }
}

impl crate::graphics::Renderer for Renderer {
    type Surface = self::Surface;
    type Pipeline = self::Pipeline;

    fn create_surface(&mut self, viewport: &Viewport) -> Self::Surface {
        Surface::new(self.display, viewport.physical_size())
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, viewport: &Viewport) {
        *surface = Surface::new(self.display, viewport.physical_size())
    }

    fn create_pipeline(&mut self, _viewport: &Viewport) -> Self::Pipeline {
        Pipeline::new(self.display)
    }

    fn perform_pipeline(
        &mut self,
        surface: &mut Self::Surface,
        pipeline: &mut Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    ) {
        let alloc_background_color = pipeline.alloc_color(&background_color);

        self.fill_rectangle(
            surface,
            &alloc_background_color,
            PhysicalRectangle::from(viewport.physical_size()),
        );

        for draw_op in pipeline.draw_ops() {
            self.process_draw_op(draw_op, surface);
        }

        self.commit(surface, viewport.physical_size());
    }

    fn update_pipeline(
        &mut self,
        pipeline: &mut Self::Pipeline,
        primitive: &Primitive,
        depth: usize,
    ) {
        pipeline.push(primitive, depth);
    }
}

impl Surface {
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

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            xlib::XFreeGC(self.display, self.gc);
            xlib::XFreePixmap(self.display, self.pixmap);
        }
    }
}
