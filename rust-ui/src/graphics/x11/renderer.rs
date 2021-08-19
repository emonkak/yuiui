use std::collections::VecDeque;
use std::ops::Add;
use std::ptr;
use x11::xlib;

use crate::base::{PhysicalRectangle, PhysicalSize};
use crate::graphics::color::Color;
use crate::graphics::renderer::Renderer;

pub struct XRenderer {
    display: *mut xlib::Display,
    window: xlib::Window,
}

pub struct DrawArea {
    display: *mut xlib::Display,
    pixmap: xlib::Pixmap,
    gc: xlib::GC,
    size: PhysicalSize,
}

#[derive(Debug)]
pub enum DrawOp {
    None,
    Batch(VecDeque<DrawOp>),
    FillRectangle(Color, PhysicalRectangle),
}

impl XRenderer {
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
        &mut self,
        draw_area: &DrawArea,
        color: &xlib::XColor,
        bounds: &PhysicalRectangle,
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

    fn commit(&mut self, draw_area: &DrawArea) {
        unsafe {
            xlib::XCopyArea(
                self.display,
                draw_area.pixmap,
                self.window,
                draw_area.gc,
                0,
                0,
                draw_area.size.width,
                draw_area.size.height,
                0,
                0,
            );
            xlib::XFlush(self.display);
        }
    }

    fn clear(&self, draw_area: &DrawArea, color: &xlib::XColor) {
        unsafe {
            xlib::XSetForeground(self.display, draw_area.gc, color.pixel);
            xlib::XFillRectangle(
                self.display,
                draw_area.pixmap,
                draw_area.gc,
                0,
                0,
                draw_area.size.width,
                draw_area.size.height,
            );
        }
    }
}

impl Renderer for XRenderer {
    type DrawArea = self::DrawArea;

    type DrawOp = DrawOp;

    fn create_draw_area(&mut self, size: PhysicalSize) -> Self::DrawArea {
        DrawArea::new(self.display, size)
    }

    fn perform_draw(
        &mut self,
        draw_area: &Self::DrawArea,
        mut draw_op: &Self::DrawOp,
        background_color: Color,
    ) {
        let alloc_background_color = self.alloc_color(&background_color);
        self.clear(draw_area, &alloc_background_color);

        let mut pending_ops = VecDeque::new();

        loop {
            match draw_op {
                DrawOp::None => {}
                DrawOp::Batch(batch_ops) => {
                    pending_ops.extend(batch_ops);
                }
                DrawOp::FillRectangle(color, bounds) => {
                    let alloc_color = self.alloc_color(color);
                    self.fill_rectangle(draw_area, &alloc_color, bounds);
                }
            }
            if let Some(pending_op) = pending_ops.pop_front() {
                draw_op = pending_op;
            } else {
                break;
            }
        }

        self.commit(draw_area);
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

            Self { display, pixmap, gc, size }
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

impl Default for DrawOp {
    fn default() -> Self {
        DrawOp::None
    }
}

impl Add for DrawOp {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (DrawOp::None, y) => y,
            (x, DrawOp::None) => x,
            (DrawOp::Batch(mut xs), DrawOp::Batch(ys)) => {
                xs.extend(ys);
                DrawOp::Batch(xs)
            }
            (DrawOp::Batch(mut xs), y) => {
                xs.push_back(y);
                DrawOp::Batch(xs)
            }
            (x, DrawOp::Batch(mut ys)) => {
                ys.push_front(x);
                DrawOp::Batch(ys)
            }
            (x, y) => {
                let mut xs = VecDeque::with_capacity(2);
                xs.push_back(x);
                xs.push_back(y);
                DrawOp::Batch(xs)
            }
        }
    }
}
