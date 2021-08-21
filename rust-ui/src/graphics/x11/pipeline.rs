use crate::graphics::Pipeline as PipelineTrait;

use super::primitive::Primitive;

#[derive(Debug)]
pub struct Pipeline {
    primitives: Vec<Primitive>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }
}

impl PipelineTrait<Primitive> for Pipeline {
    fn push(&mut self, primitive: &Primitive, _depth: usize) {
        self.primitives.push(primitive.clone());
    }
}
