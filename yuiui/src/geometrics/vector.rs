use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    fn add(self, vector: Self) -> Self {
        Self {
            dx: self.dx + vector.dx,
            dy: self.dy + vector.dy,
        }
    }
}

impl<T> Sub for Vector<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    #[inline]
    fn sub(self, vector: Self) -> Self {
        Self {
            dx: self.dx - vector.dx,
            dy: self.dy - vector.dy,
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
