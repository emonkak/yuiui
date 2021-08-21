use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Point<T = f32> {
    pub x: T,
    pub y: T,
}

pub type PhysicalPoint = Point<u32>;

impl Point<f32> {
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

impl<T> AddAssign for Point<T>
where
    T: AddAssign,
{
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl<T> Sub for Point<T>
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

impl<T> SubAssign for Point<T>
where
    T: SubAssign,
{
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}
