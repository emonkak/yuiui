use super::primitive::Primitive;

pub trait Pipeline {
    fn push(&mut self, primitive: &Primitive, depth: usize);

    #[inline]
    fn finish(&mut self) {}
}
