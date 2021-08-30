use crate::geometrics::PhysicalPoint;
use crate::support::bit_flags::BitFlags;

use super::event::EventType;
use super::keyboard::Modifier;

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub position: PhysicalPoint,
    pub button: MouseButton,
    pub buttons: BitFlags<MouseButton>,
    pub modifiers: BitFlags<Modifier>,
}

pub struct Click;

pub struct MouseUp;

pub struct MouseDown;

impl EventType for Click {
    type Event = MouseEvent;
}

impl EventType for MouseUp {
    type Event = MouseEvent;
}

impl EventType for MouseDown {
    type Event = MouseEvent;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
#[rustfmt::skip]
pub enum MouseButton {
    None    = 0b00000,
    Left    = 0b00001,
    Right   = 0b00010,
    Middle  = 0b00100,
    X1      = 0b01000,
    X2      = 0b10000,
}

impl Into<usize> for MouseButton {
    fn into(self) -> usize {
        self as usize
    }
}
