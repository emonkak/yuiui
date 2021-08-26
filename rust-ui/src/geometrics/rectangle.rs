use std::ops::{Add, Mul};

use super::{Point, Size, Vector};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rectangle<T = f32> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

pub type PhysicalRectangle = Rectangle<u32>;

impl Rectangle {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    #[inline]
    pub fn snap(self) -> Rectangle<u32> {
        Rectangle {
            x: self.x as u32,
            y: self.y as u32,
            width: self.width as u32,
            height: self.height as u32,
        }
    }

    #[inline]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);

        let width = right - left;
        let height = bottom - top;

        if width > 0.0 && height > 0.0 {
            Some(Self {
                x: left,
                y: top,
                width,
                height,
            })
        } else {
            None
        }
    }
}

impl<T> Rectangle<T> {
    #[inline]
    pub fn new(point: Point<T>, size: Size<T>) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width: size.width,
            height: size.height,
        }
    }

    #[inline]
    pub fn point(&self) -> Point<T>
    where
        T: Copy,
    {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    #[inline]
    pub fn size(&self) -> Size<T>
    where
        T: Copy,
    {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    #[inline]
    pub fn scale(&self, scale: T) -> Self
    where
        T: Copy + Mul<Output = T>,
    {
        Self {
            x: self.x * scale,
            y: self.y * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
    }

    #[inline]
    pub fn translate(&self, vector: Vector<T>) -> Self
    where
        T: Copy + Add<Output = T>,
    {
        Self {
            x: self.x + vector.dx,
            y: self.y + vector.dy,
            width: self.width,
            height: self.height,
        }
    }
}

impl<T: Default> From<Point<T>> for Rectangle<T> {
    #[inline]
    fn from(point: Point<T>) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl<T: Default> From<Size<T>> for Rectangle<T> {
    #[inline]
    fn from(size: Size<T>) -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            width: size.width,
            height: size.height,
        }
    }
}
