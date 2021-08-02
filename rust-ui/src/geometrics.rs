use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
    pub point: Point,
    pub size: Size,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub const ZERO: Self = Self {
        point: Point::ZERO,
        size: Size::ZERO,
    };
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

impl Add for Point {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Point {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl Sub for Point {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Point {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}

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

impl Add for Size {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

impl AddAssign for Size {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            width: self.width + other.width,
            height: self.height + other.height,
        };
    }
}

impl Into<Size> for &WindowSize {
    fn into(self) -> Size {
        Size {
            width: self.width as _,
            height: self.height as _,
        }
    }
}
