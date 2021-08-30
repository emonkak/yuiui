use std::os::raw::*;
use x11::xlib;

use crate::event::mouse::{MouseButton, MouseEvent};
use crate::event::keyboard::Modifier;
use crate::event::window::{WindowResizeEvent};
use crate::geometrics::{PhysicalSize, PhysicalPoint};
use crate::support::bit_flags::BitFlags;

impl From<&xlib::XButtonEvent> for MouseEvent {
    fn from(event: &xlib::XButtonEvent) -> Self {
        Self {
            position: PhysicalPoint {
                x: event.x as u32,
                y: event.y as u32,
            },
            button: to_mouse_button(event.button),
            buttons: to_mouse_buttons(event.state),
            modifiers: to_modifiers(event.state),
        }
    }
}

impl From<&xlib::XConfigureEvent> for WindowResizeEvent {
    fn from(event: &xlib::XConfigureEvent) -> Self {
        Self {
            size: PhysicalSize {
                width: event.width as _,
                height: event.height as _,
            },
        }
    }
}

fn to_mouse_button(button: c_uint) -> MouseButton {
    match button {
        xlib::Button1 => MouseButton::Left,
        xlib::Button2 => MouseButton::Right,
        xlib::Button3 => MouseButton::Middle,
        xlib::Button4 => MouseButton::X1,
        xlib::Button5 => MouseButton::X2,
        _ => MouseButton::None,
    }
}

fn to_mouse_buttons(state: c_uint) -> BitFlags<MouseButton> {
    let mut flags = BitFlags::<MouseButton>::empty();
    if xlib::Button1Mask & state != 0 {
        flags |= MouseButton::Left;
    }
    if xlib::Button2Mask & state != 0 {
        flags |= MouseButton::Right;
    }
    if xlib::Button3Mask & state != 0 {
        flags |= MouseButton::Middle;
    }
    if xlib::Button4Mask & state != 0 {
        flags |= MouseButton::X1;
    }
    if xlib::Button5Mask & state != 0 {
        flags |= MouseButton::X2;
    }
    flags
}

fn to_modifiers(state: c_uint) -> BitFlags<Modifier> {
    let mut flags = BitFlags::<Modifier>::empty();
    if xlib::Mod1Mask & state != 0 {
        flags |= Modifier::Alt;
    }
    if xlib::ShiftMask & state != 0 {
        flags |= Modifier::Shift;
    }
    if xlib::ControlMask & state != 0 {
        flags |= Modifier::Control;
    }
    if xlib::Mod4Mask & state != 0 {
        flags |= Modifier::Super;
    }
    flags
}
