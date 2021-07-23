use crate::geometrics::Rectangle;

#[derive(Debug, Default)]
pub struct PaintState {
    pub rectangle: Rectangle,
}

pub trait PaintContext<Handle> {
    fn handle(&self) -> &Handle;

    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, rectangle: &Rectangle);
}
