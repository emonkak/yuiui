use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point<T = f32> {
    pub x: T,
    pub y: T,
}

pub type PhysicalPoint = Point<u32>;

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

impl<T> Add for Point<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> Sub<Point<T>> for Point<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
