use std::os::raw::*;
use x11::xlib;

use crate::bit_flags::BitFlags;
use crate::event::mouse::{MouseButton, MouseEvent};
use crate::geometrics::Point;

#[derive(Debug)]
pub enum XEvent {
    MotionNotify(xlib::XMotionEvent),
    ButtonPress(xlib::XButtonEvent),
    ButtonRelease(xlib::XButtonEvent),
    ColormapNotify(xlib::XColormapEvent),
    EnterNotify(xlib::XCrossingEvent),
    LeaveNotify(xlib::XCrossingEvent),
    Expose(xlib::XExposeEvent),
    GraphicsExpose(xlib::XGraphicsExposeEvent),
    NoExpose(xlib::XNoExposeEvent),
    FocusIn(xlib::XFocusChangeEvent),
    FocusOut(xlib::XFocusChangeEvent),
    KeymapNotify(xlib::XKeymapEvent),
    KeyPress(xlib::XKeyEvent),
    KeyRelease(xlib::XKeyEvent),
    PropertyNotify(xlib::XPropertyEvent),
    ResizeRequest(xlib::XResizeRequestEvent),
    CirculateNotify(xlib::XCirculateEvent),
    ConfigureNotify(xlib::XConfigureEvent),
    DestroyNotify(xlib::XDestroyWindowEvent),
    GravityNotify(xlib::XGravityEvent),
    MapNotify(xlib::XMapEvent),
    ReparentNotify(xlib::XReparentEvent),
    UnmapNotify(xlib::XUnmapEvent),
    CreateNotify(xlib::XCreateWindowEvent),
    CirculateRequest(xlib::XCirculateRequestEvent),
    ConfigureRequest(xlib::XConfigureRequestEvent),
    MapRequest(xlib::XMapRequestEvent),
    ClientMessage(xlib::XClientMessageEvent),
    MappingNotify(xlib::XMappingEvent),
    SelectionClear(xlib::XSelectionClearEvent),
    SelectionNotify(xlib::XSelectionEvent),
    SelectionRequest(xlib::XSelectionRequestEvent),
    VisibilityNotify(xlib::XVisibilityEvent),
    Any(xlib::XAnyEvent),
}

impl From<&xlib::XEvent> for XEvent {
    fn from(event: &xlib::XEvent) -> XEvent {
        use self::XEvent::*;

        match event.get_type() {
            xlib::MotionNotify => MotionNotify(xlib::XMotionEvent::from(event)),
            xlib::ButtonPress => ButtonPress(xlib::XButtonEvent::from(event)),
            xlib::ButtonRelease => ButtonRelease(xlib::XButtonEvent::from(event)),
            xlib::ColormapNotify => ColormapNotify(xlib::XColormapEvent::from(event)),
            xlib::EnterNotify => EnterNotify(xlib::XCrossingEvent::from(event)),
            xlib::LeaveNotify => LeaveNotify(xlib::XCrossingEvent::from(event)),
            xlib::Expose => Expose(xlib::XExposeEvent::from(event)),
            xlib::GraphicsExpose => GraphicsExpose(xlib::XGraphicsExposeEvent::from(event)),
            xlib::NoExpose => NoExpose(xlib::XNoExposeEvent::from(event)),
            xlib::FocusIn => FocusIn(xlib::XFocusChangeEvent::from(event)),
            xlib::FocusOut => FocusOut(xlib::XFocusChangeEvent::from(event)),
            xlib::KeymapNotify => KeymapNotify(xlib::XKeymapEvent::from(event)),
            xlib::KeyPress => KeyPress(xlib::XKeyEvent::from(event)),
            xlib::KeyRelease => KeyRelease(xlib::XKeyEvent::from(event)),
            xlib::PropertyNotify => PropertyNotify(xlib::XPropertyEvent::from(event)),
            xlib::ResizeRequest => ResizeRequest(xlib::XResizeRequestEvent::from(event)),
            xlib::CirculateNotify => CirculateNotify(xlib::XCirculateEvent::from(event)),
            xlib::ConfigureNotify => ConfigureNotify(xlib::XConfigureEvent::from(event)),
            xlib::DestroyNotify => DestroyNotify(xlib::XDestroyWindowEvent::from(event)),
            xlib::GravityNotify => GravityNotify(xlib::XGravityEvent::from(event)),
            xlib::MapNotify => MapNotify(xlib::XMapEvent::from(event)),
            xlib::ReparentNotify => ReparentNotify(xlib::XReparentEvent::from(event)),
            xlib::UnmapNotify => UnmapNotify(xlib::XUnmapEvent::from(event)),
            xlib::CreateNotify => CreateNotify(xlib::XCreateWindowEvent::from(event)),
            xlib::CirculateRequest => CirculateRequest(xlib::XCirculateRequestEvent::from(event)),
            xlib::ConfigureRequest => ConfigureRequest(xlib::XConfigureRequestEvent::from(event)),
            xlib::MapRequest => MapRequest(xlib::XMapRequestEvent::from(event)),
            xlib::ClientMessage => ClientMessage(xlib::XClientMessageEvent::from(event)),
            xlib::MappingNotify => MappingNotify(xlib::XMappingEvent::from(event)),
            xlib::SelectionClear => SelectionClear(xlib::XSelectionClearEvent::from(event)),
            xlib::SelectionNotify => SelectionNotify(xlib::XSelectionEvent::from(event)),
            xlib::SelectionRequest => SelectionRequest(xlib::XSelectionRequestEvent::from(event)),
            xlib::VisibilityNotify => VisibilityNotify(xlib::XVisibilityEvent::from(event)),
            _ => Any(xlib::XAnyEvent::from(event)),
        }
    }
}

impl From<xlib::XButtonEvent> for MouseEvent {
    fn from(event: xlib::XButtonEvent) -> Self {
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
