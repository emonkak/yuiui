use x11rb::protocol::xproto;
use yuiui_support::bit_flags::BitFlags;

use crate::event::{Modifier, MouseButton, MouseEvent};
use crate::geometrics::PhysicalPoint;

impl From<xproto::ButtonPressEvent> for MouseEvent {
    fn from(event: xproto::ButtonPressEvent) -> Self {
        Self {
            position: PhysicalPoint {
                x: event.event_x as u32,
                y: event.event_y as u32,
            },
            button: to_mouse_button(event.detail),
            buttons: to_mouse_buttons(event.state),
            modifiers: to_modifiers(event.state),
        }
    }
}

fn to_mouse_button(button: u8) -> MouseButton {
    match button {
        1 => MouseButton::Left,
        2 => MouseButton::Right,
        3 => MouseButton::Middle,
        4..=7 => MouseButton::None, // scroll wheel
        8 => MouseButton::X1,
        9 => MouseButton::X2,
        _ => {
            log::warn!("Unknown mouse button code: {}", button);
            MouseButton::None
        }
    }
}

fn to_mouse_buttons(state: u16) -> BitFlags<MouseButton> {
    let mut flags = BitFlags::<MouseButton>::empty();
    if u16::from(xproto::KeyButMask::BUTTON1) & state != 0 {
        flags |= MouseButton::Left;
    }
    if u16::from(xproto::KeyButMask::BUTTON2) & state != 0 {
        flags |= MouseButton::Right;
    }
    if u16::from(xproto::KeyButMask::BUTTON3) & state != 0 {
        flags |= MouseButton::Middle;
    }
    if u16::from(xproto::KeyButMask::BUTTON4) & state != 0 {
        flags |= MouseButton::X1;
    }
    if u16::from(xproto::KeyButMask::BUTTON5) & state != 0 {
        flags |= MouseButton::X2;
    }
    flags
}

fn to_modifiers(state: u16) -> BitFlags<Modifier> {
    let mut flags = BitFlags::<Modifier>::empty();
    if u16::from(xproto::KeyButMask::MOD1) & state != 0 {
        flags |= Modifier::Alt;
    }
    if u16::from(xproto::KeyButMask::SHIFT) & state != 0 {
        flags |= Modifier::Shift;
    }
    if u16::from(xproto::KeyButMask::CONTROL) & state != 0 {
        flags |= Modifier::Control;
    }
    if u16::from(xproto::KeyButMask::MOD4) & state != 0 {
        flags |= Modifier::Super;
    }
    flags
}
