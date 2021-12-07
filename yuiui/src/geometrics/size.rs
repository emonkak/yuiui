use crate::geometrics::RectOutsets;
use std::ops::Add;

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

impl<T> Size<T>
where
    T: Add<Output = T> + Copy,
{
    pub fn inflate(&self, outsets: RectOutsets<T>) -> Self {
        Self {
            width: self.width + outsets.left + outsets.right,
            height: self.height + outsets.top + outsets.bottom,
        }
    }
}
