use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Vector<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T> Vector<T> {
    #[inline]
    pub fn scale(&self, scale: T) -> Self
    where
        T: Mul<Output = T> + Copy,
    {
        Self {
            x: self.x * scale,
            y: self.y * scale,
        }
    }
}

impl<T> Add for Vector<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    #[inline]
    fn add(self, b: Self) -> Self {
        Self {
            x: self.x + b.x,
            y: self.y + b.y,
        }
    }
}

impl<T> Sub for Vector<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    #[inline]
    fn sub(self, b: Self) -> Self {
        Self {
            x: self.x - b.x,
            y: self.y - b.y,
        }
    }
}

impl<T> From<[T; 2]> for Vector<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self { x, y }
    }
}

impl<T> From<Vector<T>> for [T; 2] {
    fn from(other: Vector<T>) -> Self {
        [other.x, other.y]
    }
}
