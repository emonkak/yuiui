use crate::geometrics::{Rectangle, Transform};
use super::quad::Quad;
use super::text::Text;

#[derive(Debug)]
pub struct Layer {
    pub bounds: Option<Rectangle>,
    pub transform: Transform,
    pub quads: Vec<Quad>,
    pub texts: Vec<Text>,
}

impl Layer {
    pub fn new(bounds: Option<Rectangle>, transform: Transform) -> Self {
        Self {
            bounds,
            transform,
            quads: Vec::new(),
            texts: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.quads.is_empty() && self.texts.is_empty()
    }
}
