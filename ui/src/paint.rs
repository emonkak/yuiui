use geometrics::{Rectangle};

pub trait Painter<Handle> {
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, handle: &Handle, rectangle: &Rectangle);
}

pub struct PaintContext<Handle> {
    painter: Box<dyn Painter<Handle>>,
}

impl<Handle> PaintContext<Handle> {
    pub fn new(painter: impl Painter<Handle> + 'static) -> Self {
        PaintContext {
            painter: Box::new(painter)
        }
    }

    pub fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle) {
        self.painter.fill_rectangle(color, rectangle);
    }

    pub fn commit(&mut self, handle: &Handle, rectangle: &Rectangle) {
        self.painter.commit(handle, rectangle);
    }
}
