use yuiui_support::bit_flags::BitFlags;

use super::keyboard::Modifier;
use crate::geometrics::PhysicalPoint;

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub position: PhysicalPoint,
    pub button: MouseButton,
    pub buttons: BitFlags<MouseButton>,
    pub modifiers: BitFlags<Modifier>,
}

pub struct MouseUp(pub MouseEvent);

pub struct MouseDown(pub MouseEvent);

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(usize)]
#[rustfmt::skip]
pub enum MouseButton {
    None   = 0,
    Left   = 1 << 1,
    Right  = 1 << 2,
    Middle = 1 << 3,
    X1     = 1 << 4,
    X2     = 1 << 5,
}

impl Into<usize> for MouseButton {
    fn into(self) -> usize {
        self as usize
    }
}
