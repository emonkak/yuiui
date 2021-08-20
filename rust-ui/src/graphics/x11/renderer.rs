use std::ptr;
use x11::xlib;

use crate::base::{PhysicalRectangle, PhysicalSize};
use crate::graphics::color::Color;
use crate::graphics::renderer::Renderer as RendererTrait;
use crate::graphics::viewport::Viewport;

use super::pipeline::{DrawOp, Pipeline};

pub struct Renderer {
    display: *mut xlib::Display,
    window: xlib::Window,
}

pub struct View {
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

    fn fill_rectangle(&mut self, view: &View, color: &xlib::XColor, bounds: PhysicalRectangle) {
        unsafe {
            xlib::XSetForeground(self.display, view.gc, color.pixel);
            xlib::XFillRectangle(
                self.display,
                view.pixmap,
                view.gc,
                bounds.x as _,
                bounds.y as _,
                bounds.width as _,
                bounds.height as _,
            );
        }
    }

    fn commit(&mut self, draw_area: &View, size: PhysicalSize) {
        unsafe {
            xlib::XCopyArea(
                self.display,
                draw_area.pixmap,
                self.window,
                draw_area.gc,
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
}

impl RendererTrait for Renderer {
    type View = self::View;

    type Pipeline = Pipeline;

    fn create_view(&mut self, viewport: &Viewport) -> Self::View {
        View::new(self.display, viewport.physical_size())
    }

    fn create_pipeline(&mut self, _viewport: &Viewport) -> Self::Pipeline {
        Pipeline::new()
    }

    fn perform_pipeline(
        &mut self,
        view: &mut Self::View,
        pipeline: &Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    ) {
        let alloc_background_color = self.alloc_color(&background_color);

        self.fill_rectangle(
            view,
            &alloc_background_color,
            PhysicalRectangle::from(viewport.physical_size()),
        );

        for draw_op in &pipeline.draw_ops {
            match draw_op {
                DrawOp::FillRectangle(color, bounds) => {
                    let alloc_color = self.alloc_color(color);
                    self.fill_rectangle(view, &alloc_color, *bounds);
                }
            }
        }

        self.commit(view, viewport.physical_size());
    }
}

impl View {
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

impl Drop for View {
    fn drop(&mut self) {
        unsafe {
            xlib::XFreeGC(self.display, self.gc);
            xlib::XFreePixmap(self.display, self.pixmap);
        }
    }
}
