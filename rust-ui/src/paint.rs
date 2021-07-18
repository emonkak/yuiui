use crate::geometrics::Rectangle;

pub trait PaintContext<Handle> {
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, handle: &Handle, rectangle: &Rectangle);
}
