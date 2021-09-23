#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size<T = f32> {
    pub width: T,
    pub height: T,
}

pub type PhysicalSize = Size<u32>;

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };
}
