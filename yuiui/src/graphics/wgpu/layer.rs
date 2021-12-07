use super::quad::Quad;
use super::text::Text;
use crate::geometrics::{Rect, Transform};

#[derive(Debug)]
pub struct Layer {
    pub bounds: Option<Rect>,
    pub transform: Transform,
    pub quads: Vec<Quad>,
    pub texts: Vec<Text>,
}

impl Layer {
    pub fn new(bounds: Option<Rect>, transform: Transform) -> Self {
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
