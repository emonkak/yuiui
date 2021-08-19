use std::os::raw::*;
use x11::xlib;

use crate::base::{PhysicalSize, Point};
use crate::bit_flags::BitFlags;
use crate::event::mouse::{MouseButton, MouseEvent};
use crate::event::window::WindowResizeEvent;

impl From<&xlib::XButtonEvent> for MouseEvent {
    fn from(event: &xlib::XButtonEvent) -> Self {
        Self {
            point: Point {
                x: event.x as _,
                y: event.y as _,
            },
            button: to_mouse_button(event.button),
            buttons: to_mouse_buttons(event.state),
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
    let mut flags = BitFlags::<MouseButton>::new();
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
