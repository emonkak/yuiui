#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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

    #[inline]
    pub fn expand(&self) -> Self {
        Size {
            width: self.width.abs().ceil().copysign(self.width),
            height: self.height.abs().ceil().copysign(self.height),
        }
    }
}

impl From<Size<u32>> for Size<f32> {
    #[inline]
    fn from(size: Size<u32>) -> Self {
        Self {
            width: size.width as _,
            height: size.height as _,
        }
    }
}

impl From<Size<f32>> for Size<u32> {
    #[inline]
    fn from(size: Size<f32>) -> Self {
        Self {
            width: size.width as _,
            height: size.height as _,
        }
    }
}
