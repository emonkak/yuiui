use super::{PhysicalSize, Size, Transform};

#[derive(Debug, Clone)]
pub struct Viewport {
    physical_size: PhysicalSize,
    logical_size: Size,
    scale_factor: f32,
}

impl Viewport {
    #[inline]
    pub fn from_physical(physical_size: PhysicalSize, scale_factor: f32) -> Viewport {
        Viewport {
            physical_size,
            logical_size: Size {
                width: (physical_size.width as f32 / scale_factor),
                height: (physical_size.height as f32 / scale_factor),
            },
            scale_factor,
        }
    }

    #[inline]
    pub fn from_logical(logical_size: Size, scale_factor: f32) -> Viewport {
        Viewport {
            physical_size: PhysicalSize {
                width: (logical_size.width * scale_factor).round() as u32,
                height: (logical_size.height * scale_factor).round() as u32,
            },
            logical_size,
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
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    #[inline]
    pub fn projection(&self) -> Transform {
        Transform::orthographic(self.physical_size.width, self.physical_size.height)
    }
}
