use std::ops::Add;

use crate::geometrics::{Rectangle, Transform};
use crate::graphics::{Background, Color};
use crate::text::{FontDescriptor, HorizontalAlign, VerticalAlign};

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    None,
    Batch(Vec<Primitive>),
    Transform(Transform, Box<Primitive>),
    Clip(Rectangle, Box<Primitive>),
    Quad {
        bounds: Rectangle,
        background: Background,
        border_radius: f32,
        border_width: f32,
        border_color: Color,
    },
    Text {
        bounds: Rectangle,
        content: String,
        color: Color,
        font: FontDescriptor,
        font_size: f32,
        horizontal_align: HorizontalAlign,
        vertical_align: VerticalAlign,
    },
}

impl Default for Primitive {
    fn default() -> Self {
        Self::None
    }
}

impl Add for Primitive {
    type Output = Primitive;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::None, y) => y,
            (x, Self::None) => x,
            (Self::Batch(mut xs), Self::Batch(ys)) => {
                xs.extend(ys);
                Self::Batch(xs)
            }
            (Self::Batch(mut xs), y) => {
                xs.push(y);
                Self::Batch(xs)
            }
            (x, Self::Batch(ys)) => {
                let mut xs = vec![x];
                xs.extend(ys);
                Self::Batch(xs)
            }
            (x, y) => Self::Batch(vec![x, y]),
        }
    }
}
