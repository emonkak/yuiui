use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rectangle {
    pub point: Point,
    pub size: Size,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub const ZERO: Self = Self { point: Point::ZERO, size: Size::ZERO };
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
            y: self.y + other.y
        }
    }
}

impl AddAssign for Point {
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
            y: self.y - other.y
        }
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}

impl Size {
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };
}

impl Add for Size {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height
        }
    }
}

impl AddAssign for Size {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            width: self.width + other.width,
            height: self.height + other.height
        };
    }
}
