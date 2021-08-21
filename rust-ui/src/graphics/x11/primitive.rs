use crate::geometrics::PhysicalRectangle;
use crate::graphics::Color;

#[derive(Clone, Debug)]
pub enum Primitive {
    None,
    Batch(Vec<Primitive>),
    FillRectangle(Color, PhysicalRectangle),
}

impl Default for Primitive {
    fn default() -> Self {
        Primitive::None
    }
}
