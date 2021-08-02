use std::mem;
use x11::xlib;

pub unsafe fn get_window_rectangle(display: *mut xlib::Display, window: xlib::Window) -> (i32, i32, u32, u32) {
    let mut attributes: xlib::XWindowAttributes = mem::MaybeUninit::zeroed().assume_init();
    xlib::XGetWindowAttributes(display, window, &mut attributes);
    (attributes.x, attributes.y, attributes.width as _, attributes.height as _)
}

pub unsafe fn create_window(display: *mut xlib::Display, width: u32, height: u32) -> xlib::Window {
    let screen = xlib::XDefaultScreenOfDisplay(display);
    let screen_number = xlib::XScreenNumberOfScreen(screen);
    let root = xlib::XRootWindowOfScreen(screen);

    let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
    attributes.background_pixel = xlib::XWhitePixel(display, screen_number);

    let window = xlib::XCreateWindow(
        display,
        root,
        0,
        0,
        width,
        height,
        0,
        xlib::CopyFromParent,
        xlib::InputOutput as u32,
        xlib::CopyFromParent as *mut xlib::Visual,
        xlib::CWBackPixel,
        &mut attributes,
    );

    window
}
