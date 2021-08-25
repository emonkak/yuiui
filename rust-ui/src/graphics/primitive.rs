use crate::geometrics::{Rectangle, Vector};
use crate::graphics::{Background, Color};
use crate::text::{FontDescriptor, HorizontalAlign, VerticalAlign};

#[derive(Debug, Clone)]
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
