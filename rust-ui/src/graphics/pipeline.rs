use super::primitive::Primitive;

pub trait Pipeline {
    fn push(&mut self, primitive: &Primitive, depth: usize);

    fn finish(&mut self) {}
}
