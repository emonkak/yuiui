use geometrics::{Rectangle};

pub trait Painter<Window> {
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, window: &Window, rectangle: &Rectangle);
}

pub struct PaintContext<Window> {
    painter: Box<dyn Painter<Window>>,
}

impl<Window> PaintContext<Window> {
    pub fn new(painter: impl Painter<Window> + 'static) -> Self {
        PaintContext {
            painter: Box::new(painter)
        }
    }

    pub fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle) {
        self.painter.fill_rectangle(color, rectangle);
    }

    pub fn commit(&mut self, window: &Window, rectangle: &Rectangle) {
        self.painter.commit(window, rectangle);
    }
}
