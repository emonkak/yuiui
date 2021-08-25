use crate::geometrics::{PhysicalSize, Size};

use super::Transformation;

#[derive(Debug, Clone)]
pub struct Viewport {
    physical_size: PhysicalSize,
    logical_size: Size,
    scale_factor: f64,
}

impl Viewport {
    #[inline]
    pub fn new(physical_size: PhysicalSize, scale_factor: f64) -> Viewport {
        Viewport {
            physical_size,
            logical_size: Size {
                width: (physical_size.width as f64 / scale_factor) as f32,
                height: (physical_size.height as f64 / scale_factor) as f32,
            },
            scale_factor,
        }
    }

    #[inline]
    pub fn physical_size(&self) -> PhysicalSize {
        self.physical_size
    }

    #[inline]
    pub fn logical_size(&self) -> Size {
        self.logical_size
    }

    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    #[inline]
    pub fn projection(&self) -> Transformation {
        Transformation::orthographic(self.physical_size.width, self.physical_size.height)
    }
}
