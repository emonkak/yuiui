use x11::xlib;

use crate::geometrics::PhysicalRectangle;
use crate::graphics::{Background, Color, Primitive};

#[derive(Debug)]
pub struct Pipeline {
    display: *mut xlib::Display,
    draw_ops: Vec<DrawOp>,
}

#[derive(Debug)]
pub enum DrawOp {
    FillRectangle(xlib::XColor, PhysicalRectangle),
}

impl Pipeline {
    pub fn new(display: *mut xlib::Display) -> Self {
        Self {
            display,
            draw_ops: Vec::new(),
        }
    }

    pub fn draw_ops(&self) -> &[DrawOp] {
        &self.draw_ops
    }

    pub fn alloc_color(&self, color: &Color) -> xlib::XColor {
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

    pub fn push(&mut self, primitive: &Primitive, depth: usize) {
        match primitive {
            Primitive::Batch(primitives) => {
                for primitive in primitives {
                    self.push(primitive, depth)
                }
            }
            Primitive::Transform(_) => {
                // TODO:
            }
            Primitive::Clip(_bounds) => {
                // TODO:
            }
            Primitive::Quad {
                bounds, background, ..
            } => {
                let background_color = match background {
                    Background::Color(color) => self.alloc_color(color),
                };
                self.draw_ops
                    .push(DrawOp::FillRectangle(background_color, (*bounds).into()));
            }
            Primitive::Text { .. } => {
                // TODO:
            }
        }
    }
}
