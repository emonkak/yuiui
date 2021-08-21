use crate::geometrics::{PhysicalSize, Size};

use super::Transformation;

#[derive(Debug, Clone)]
pub struct Viewport {
    physical_size: PhysicalSize,
    logical_size: Size,
    scale_factor: f64,
    projection: Transformation,
}

impl Viewport {
    pub fn new(size: PhysicalSize, scale_factor: f64) -> Viewport {
        Viewport {
            physical_size: size,
            logical_size: Size {
                width: (size.width as f64 / scale_factor) as f32,
                height: (size.height as f64 / scale_factor) as f32,
            },
            scale_factor,
            projection: Transformation::orthographic(size.width, size.height),
        }
    }

    pub fn physical_size(&self) -> PhysicalSize {
        self.physical_size
    }

    pub fn logical_size(&self) -> Size {
        self.logical_size
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn projection(&self) -> Transformation {
        self.projection
    }
}
