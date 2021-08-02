pub mod x11;

use crate::geometrics::Rectangle;

pub trait GeneralPainter {
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, rectangle: &Rectangle);
}
