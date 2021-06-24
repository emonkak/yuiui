use std::ops::{Add, Sub};

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

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Point {
    pub fn offset(&self, offset: Point) -> Self {
        Self {
            x: self.x + offset.x,
            y: self.y + offset.y,
        }
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }
}

impl Add for Size {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height
        }
    }
}

impl Sub for Size {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            width: self.width - other.width,
            height: self.height - other.height
        }
    }
}
