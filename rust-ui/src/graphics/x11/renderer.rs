use std::ptr;
use x11::xlib;

use crate::base::{PhysicalRectangle, PhysicalSize};
use crate::graphics::color::Color;
use crate::graphics::renderer::{Pipeline as PipelineTrait, Renderer as RendererTrait};
use crate::graphics::viewport::Viewport;

#[derive(Debug)]
pub struct Renderer {
    display: *mut xlib::Display,
    window: xlib::Window,
}

#[derive(Debug)]
pub struct DrawArea {
    display: *mut xlib::Display,
    pixmap: xlib::Pixmap,
    gc: xlib::GC,
}

#[derive(Debug)]
pub struct Pipeline {
    pub(crate) primitives: Vec<Primitive>,
}

#[derive(Clone, Debug)]
pub enum Primitive {
    None,
    Batch(Vec<Primitive>),
    FillRectangle(Color, PhysicalRectangle),
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

    fn fill_rectangle(
        &self,
        draw_area: &DrawArea,
        color: &xlib::XColor,
        bounds: PhysicalRectangle,
    ) {
        unsafe {
            xlib::XSetForeground(self.display, draw_area.gc, color.pixel);
            xlib::XFillRectangle(
                self.display,
                draw_area.pixmap,
                draw_area.gc,
                bounds.x as _,
                bounds.y as _,
                bounds.width as _,
                bounds.height as _,
            );
        }
    }

    fn commit(&self, draw_area: &DrawArea, size: PhysicalSize) {
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

    fn process_primitives(&self, draw_area: &DrawArea, primitives: &[Primitive]) {
        for primitive in primitives {
            match primitive {
                Primitive::None => {}
                Primitive::Batch(primitives) => {
                    self.process_primitives(draw_area, primitives);
                }
                Primitive::FillRectangle(color, bounds) => {
                    let alloc_color = self.alloc_color(color);
                    self.fill_rectangle(draw_area, &alloc_color, *bounds);
                }
            }
        }
    }
}

impl RendererTrait for Renderer {
    type DrawArea = self::DrawArea;
    type Primitive = self::Primitive;
    type Pipeline = self::Pipeline;

    fn create_draw_area(&mut self, viewport: &Viewport) -> Self::DrawArea {
        DrawArea::new(self.display, viewport.physical_size())
    }

    fn create_pipeline(&mut self, _viewport: &Viewport) -> Self::Pipeline {
        Pipeline::new()
    }

    fn perform_pipeline(
        &mut self,
        draw_area: &mut Self::DrawArea,
        pipeline: &Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    ) {
        let alloc_background_color = self.alloc_color(&background_color);

        self.fill_rectangle(
            draw_area,
            &alloc_background_color,
            PhysicalRectangle::from(viewport.physical_size()),
        );

        self.process_primitives(draw_area, &pipeline.primitives);

        self.commit(draw_area, viewport.physical_size());
    }
}

impl DrawArea {
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

impl Drop for DrawArea {
    fn drop(&mut self) {
        unsafe {
            xlib::XFreeGC(self.display, self.gc);
            xlib::XFreePixmap(self.display, self.pixmap);
        }
    }
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }
}

impl PipelineTrait<Primitive> for Pipeline {
    fn push(&mut self, primitive: &Primitive) {
        self.primitives.push(primitive.clone());
    }
}

impl Default for Primitive {
    fn default() -> Self {
        Primitive::None
    }
}