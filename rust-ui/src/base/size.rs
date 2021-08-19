use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Size<T = f32> {
    pub width: T,
    pub height: T,
}

pub type PhysicalSize = Size<u32>;

impl Size<f32> {
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

impl<T> Add for Size<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

impl<T> AddAssign for Size<T>
where
    T: AddAssign,
{
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.width += other.width;
        self.height += other.height;
    }
}

impl<T> Sub for Size<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            width: self.width - other.width,
            height: self.height - other.height,
        }
    }
}

impl<T> SubAssign for Size<T>
where
    T: SubAssign,
{
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.width -= other.width;
        self.height -= other.height;
    }
}
