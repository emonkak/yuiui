use glam::{Mat4, Vec3};
use std::ops::Mul;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform(Mat4);

impl Transform {
    pub const IDENTITY: Self = Self(Mat4::IDENTITY);

    #[rustfmt::skip]
    #[inline]
    pub fn orthographic(width: u32, height: u32) -> Self {
        Self(Mat4::orthographic_rh_gl(
            0.0, width as f32,
            height as f32, 0.0,
            -1.0, 1.0
        ))
    }

    #[inline]
    pub fn translation(x: f32, y: f32) -> Self {
        Self(Mat4::from_translation(Vec3::new(x, y, 0.0)))
    }

    #[inline]
    pub fn scale(x: f32, y: f32) -> Self {
        Self(Mat4::from_scale(Vec3::new(x, y, 1.0)))
    }

    #[inline]
    pub fn rotation(angle: f32) -> Self {
        Self(Mat4::from_rotation_z(angle))
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }
}

impl Mul for Transform {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl AsRef<[f32; 16]> for Transform {
    fn as_ref(&self) -> &[f32; 16] {
        self.0.as_ref()
    }
}

impl From<Transform> for [f32; 16] {
    fn from(transform: Transform) -> Self {
        *transform.0.as_ref()
    }
}
