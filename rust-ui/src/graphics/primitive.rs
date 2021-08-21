use crate::geometrics::{Rectangle, Vector};
use crate::graphics::{Background, Color};

#[derive(Debug)]
pub enum Primitive {
    Batch(Vec<Primitive>),
    Translate(Vector),
    Clip(Rectangle),
    Quad {
        bounds: Rectangle,
        background: Background,
        border_radius: f32,
        border_width: f32,
        border_color: Color,
    },
}
