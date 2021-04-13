use std::fs;
use std::io;
use std::mem;
use std::os::raw::*;
use std::ptr;
use x11::xlib;

#[inline]
pub fn new_atom(display: *mut xlib::Display, null_terminated_name: &str) -> xlib::Atom {
    assert!(null_terminated_name.chars().last().map_or(false, |c| c == '\0'));
    unsafe {
        xlib::XInternAtom(display, null_terminated_name.as_ptr() as *const c_char, xlib::False)
    }
}

#[inline]
pub fn send_client_message(display: *mut xlib::Display, destination: xlib::Window, window: xlib::Window, message_type: xlib::Atom, data: xlib::ClientMessageData) -> bool {
    let mut client_message_event = xlib::XEvent::from(xlib::XClientMessageEvent {
        type_: xlib::ClientMessage,
        serial: 0,
        send_event: xlib::True,
        display,
        window,
        message_type,
        format: 32,
        data,
    });

    unsafe {
        xlib::XSendEvent(
            display,
            destination,
            xlib::False,
            0xffffff,
            &mut client_message_event
        ) == xlib::True
    }
}

#[inline]
pub fn get_window_property<T: Sized, const N: usize>(display: *mut xlib::Display, window: xlib::Window, property_atom: xlib::Atom) -> Option<Box<[T; N]>> {
    let mut actual_type: xlib::Atom = 0;
    let mut actual_format: i32 = 0;
    let mut nitems: u64 = 0;
    let mut bytes_after: u64 = 0;
    let mut prop: *mut u8 = ptr::null_mut();

    let result = unsafe {
        xlib::XGetWindowProperty(
            display,
            window,
            property_atom,
            0,
            N as c_long,
            xlib::False,
            xlib::AnyPropertyType as u64,
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut prop
        )
    };

    if actual_format == 32 {
        actual_format = 64;
    }

    if result != xlib::Success.into()
        || actual_format != (mem::size_of::<T>() * 8) as i32
        || nitems != N as c_ulong
        || prop.is_null() {
        println!("format: {} {}", actual_format, mem::size_of::<T>() * 8);
        println!("items: {} {}", nitems, N);
        println!("prop: {:?}", prop);
        return None;
    }

    unsafe {
        Some(Box::from_raw(prop.cast()))
    }
}

#[inline]
pub fn get_process_name(pid: u32) -> io::Result<String> {
    let path = format!("/proc/{}/cmdline", pid);
    let bytes = fs::read(path)?;
    let null_position = bytes.iter().position(|byte| *byte == 0);
    let name = String::from_utf8_lossy(&bytes[0..null_position.unwrap_or(0)]).into_owned();
    Ok(name)
}

#[inline]
pub fn resize_window(display: *mut xlib::Display, window: xlib::Window, width: u32, height: u32) {
    let mut size_hints: xlib::XSizeHints = unsafe { mem::MaybeUninit::uninit().assume_init() };
    size_hints.flags = xlib::PSize;
    size_hints.width = width as i32;
    size_hints.height = height as i32;

    unsafe {
        xlib::XSetWMNormalHints(display, window, &mut size_hints);
        xlib::XResizeWindow(display, window, width, height);
    }
}

#[inline]
pub fn emit_button_event(display: *mut xlib::Display, window: xlib::Window, event_type: c_int, event_mask: c_long, button: c_uint, button_mask: c_uint, x: c_int, y: c_int) -> bool {
    unsafe {
        let screen = xlib::XDefaultScreen(display);
        let root = xlib::XRootWindow(display, screen);

        let mut event = xlib::XEvent::from(xlib::XButtonEvent {
            type_: event_type,
            serial: 0,
            send_event: xlib::True,
            display: display,
            window,
            root,
            subwindow: 0,
            time: xlib::CurrentTime,
            x,
            y,
            x_root: 0,
            y_root: 0,
            state: button_mask,
            button,
            same_screen: xlib::True,
        });

        xlib::XSendEvent(display, window, xlib::True, event_mask, &mut event) == xlib::True
    }
}

#[inline]
pub fn emit_crossing_event(display: *mut xlib::Display, window: xlib::Window, event_type: c_int, event_mask: c_long, focus: xlib::Bool) -> bool {
    unsafe {
        let screen = xlib::XDefaultScreen(display);
        let root = xlib::XRootWindow(display, screen);

        let mut event = xlib::XEvent::from(xlib::XCrossingEvent {
            type_: event_type,
            serial: 0,
            send_event: xlib::True,
            display: display,
            window,
            root,
            subwindow: 0,
            time: xlib::CurrentTime,
            x: 0,
            y: 0,
            x_root: 0,
            y_root: 0,
            mode: xlib::NotifyNormal,
            detail: xlib::NotifyAncestor,
            same_screen: xlib::True,
            focus,
            state: 0,
        });

        xlib::XSendEvent(display, window, xlib::True, event_mask, &mut event) == xlib::True
    }
}
