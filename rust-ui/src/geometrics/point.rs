use std::ops::{Add, Sub};

use super::Vector;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Point<T = f32> {
    pub x: T,
    pub y: T,
}

pub type PhysicalPoint = Point<u32>;

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

impl From<Point<u32>> for Point<f32> {
    #[inline]
    fn from(point: Point<u32>) -> Self {
        Self {
            x: point.x as _,
            y: point.y as _,
        }
    }
}

impl From<Point<f32>> for Point<u32> {
    #[inline]
    fn from(point: Point<f32>) -> Self {
        Self {
            x: point.x as _,
            y: point.y as _,
        }
    }
}

impl<T> From<Point<T>> for Vector<T> {
    #[inline]
    fn from(point: Point<T>) -> Vector<T> {
        Vector {
            dx: point.x,
            dy: point.y,
        }
    }
}

impl<T> Add<Vector<T>> for Point<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    #[inline]
    fn add(self, vector: Vector<T>) -> Self {
        Self {
            x: self.x + vector.dx,
            y: self.y + vector.dy,
        }
    }
}

impl<T> Sub<Vector<T>> for Point<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    #[inline]
    fn sub(self, vector: Vector<T>) -> Self {
        Self {
            x: self.x - vector.dx,
            y: self.y - vector.dy,
        }
    }
}

impl<T> Sub<Point<T>> for Point<T>
where
    T: Sub<Output = T>,
{
    type Output = Vector<T>;

    fn sub(self, other: Point<T>) -> Vector<T> {
        Vector {
            dx: self.x - other.x,
            dy: self.y - other.y,
        }
    }
}
