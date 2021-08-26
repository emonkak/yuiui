use std::os::raw::*;
use x11::xlib;

pub unsafe fn install_error_handler(
) -> Option<unsafe extern "C" fn(*mut xlib::Display, *mut xlib::XErrorEvent) -> c_int> {
    xlib::XSetErrorHandler(Some(handle))
}

unsafe extern "C" fn handle(display: *mut xlib::Display, error: *mut xlib::XErrorEvent) -> c_int {
    let error_message = x11_get_error_message(display, (*error).error_code as i32);
    let request_message = x11_get_request_description(display, (*error).request_code as i32);
    println!(
        "XError: {} (request: {}, resource: {})",
        error_message,
        request_message,
        (*error).resourceid
    );
    0
}

unsafe fn x11_get_error_message(display: *mut xlib::Display, error_code: i32) -> String {
    let mut message = [0 as u8; 255];

    xlib::XGetErrorText(
        display,
        error_code,
        message.as_mut_ptr() as *mut i8,
        message.len() as i32,
    );

    null_terminated_bytes_to_string(&message)
}

unsafe fn x11_get_request_description(display: *mut xlib::Display, request_code: i32) -> String {
    let mut message = [0 as u8; 255];
    let request_type = format!("{}\0", request_code.to_string());

    xlib::XGetErrorDatabaseText(
        display,
        "XRequest\0".as_ptr() as *const c_char,
        request_type.as_ptr() as *const c_char,
        "Unknown\0".as_ptr() as *const c_char,
        message.as_mut_ptr() as *mut i8,
        message.len() as i32,
    );

    null_terminated_bytes_to_string(&message)
}

fn null_terminated_bytes_to_string(cs: &[u8]) -> String {
    let cs = match cs.iter().position(|&c| c == b'\0') {
        Some(null_pos) => &cs[..null_pos],
        _ => cs,
    };
    String::from_utf8_lossy(&cs).into_owned()
}