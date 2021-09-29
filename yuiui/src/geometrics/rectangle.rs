use std::ops::{Add, Mul};

use super::{Point, Size, Vector};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub fn snap(self) -> PhysicalRectangle {
        Rectangle {
            x: self.x as u32,
            y: self.y as u32,
            width: self.width as u32,
            height: self.height as u32,
        }
    }

    #[inline]
    pub fn intersection(&self, other: Self) -> Option<Self> {
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

    #[inline]
    pub fn union(&self, other: Self) -> Self {
        let left = self.x.min(other.x);
        let right = (self.x + self.width).max(other.x + other.width);
        let top = self.y.min(other.y);
        let bottom = (self.y + self.height).max(other.y + other.height);
        Self {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
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
    pub fn from_point(point: Point<T>) -> Self
    where
        T: Default,
    {
        Self {
            x: point.x,
            y: point.y,
            width: T::default(),
            height: T::default(),
        }
    }

    #[inline]
    pub fn from_size(size: Size<T>) -> Self
    where
        T: Default,
    {
        Self {
            x: T::default(),
            y: T::default(),
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
    pub fn contains(&self, point: Point<T>) -> bool
    where
        T: Add<Output = T> + Copy + Ord,
    {
        self.x <= point.x
            && point.x <= self.x + self.width
            && self.y <= point.y
            && point.y <= self.y + self.height
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
}
