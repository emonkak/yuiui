use std::ops::{Add, Mul, Sub};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vector<T = f32> {
    pub dx: T,
    pub dy: T,
}

impl Vector {
    pub const ZERO: Self = Self { dx: 0.0, dy: 0.0 };
}

impl<T> Add for Vector<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            dx: self.dx + other.dx,
            dy: self.dy + other.dy,
        }
    }
}

impl<T> Sub for Vector<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            dx: self.dx - other.dx,
            dy: self.dy - other.dy,
        }
    }
}

impl<T> Mul<T> for Vector<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    #[inline]
    fn mul(self, scale: T) -> Self {
        Self {
            dx: self.dx * scale,
            dy: self.dy * scale,
        }
    }
}
