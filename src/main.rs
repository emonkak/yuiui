extern crate keytray;
extern crate libc;
extern crate nix;
extern crate x11;

use keytray::event_handler::EventHandler;
use keytray::signal_handler;
use keytray::tray::Tray;
use nix::sys::epoll;
use nix::sys::signal;
use nix::unistd;
use std::env;
use std::ffi::CString;
use std::mem;
use std::os::raw::*;
use std::os::unix::io::RawFd;
use std::ptr;
use x11::xlib;

fn main() {
    unsafe {
        xlib::XSetErrorHandler(Some(x11_error_handler));
    }

    let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
    if display.is_null() {
        panic!(
            "No display found at {}",
            env::var("DISPLAY").unwrap_or_default()
        );
    }

    let epoll_fd = epoll::epoll_create().unwrap();
    let mut epoll_events = [epoll::EpollEvent::empty(); 2];

    let x11_fd = unsafe { xlib::XConnectionNumber(display) as RawFd };
    let mut x11_ev = epoll::EpollEvent::new(epoll::EpollFlags::EPOLLIN, x11_fd as u64);
    epoll::epoll_ctl(epoll_fd, epoll::EpollOp::EpollCtlAdd, x11_fd, Some(&mut x11_ev)).unwrap();

    let signal_fd = signal_handler::install(&[signal::Signal::SIGINT]).unwrap();
    let mut signal_ev = epoll::EpollEvent::new(epoll::EpollFlags::EPOLLIN, signal_fd as u64);
    epoll::epoll_ctl(epoll_fd, epoll::EpollOp::EpollCtlAdd, signal_fd, Some(&mut signal_ev)).unwrap();

    let mut tray = Tray::new(display);
    tray.acquire_tray_selection();
    tray.show();

    unsafe {
        xlib::XFlush(display);
    }

    let mut signal_buffer = [0; mem::size_of::<c_int>()];
    let mut event: xlib::XEvent = unsafe { mem::MaybeUninit::uninit().assume_init() };

    'outer: loop {
        let num_fds = epoll::epoll_wait(epoll_fd, &mut epoll_events, -1).unwrap_or(0);

        for i in 0..num_fds {
            let changed_fd = epoll_events[i].data() as RawFd;
            if changed_fd == x11_fd {
                unsafe {
                    xlib::XNextEvent(display, &mut event);
                }

                if !tray.handle_event(event) {
                    break 'outer;
                }
            } else if changed_fd == signal_fd {
                if let Ok(_) = unistd::read(signal_fd, &mut signal_buffer[..]) {
                    break 'outer;
                }
            }
        }
    }

    mem::drop(tray);

    unsafe {
        xlib::XCloseDisplay(display);
    }
}

fn x11_get_error_message(display: *mut xlib::Display, error_code: i32) -> CString {
    let mut message = [0 as u8; 255];

    unsafe {
        xlib::XGetErrorText(
            display,
            error_code,
            message.as_mut_ptr() as *mut i8,
            message.len() as i32
        );
    }

    raw_to_cstring(message)
}

fn x11_get_request_description(display: *mut xlib::Display, request_code: i32) -> CString {
    let mut message = [0 as u8; 255];

    let request_name = CString::new("XRequest").unwrap();
    let request_type = CString::new(request_code.to_string()).unwrap();
    let default_string = CString::new("Unknown").unwrap();

    unsafe {
        xlib::XGetErrorDatabaseText(
            display,
            request_name.as_ptr(),
            request_type.as_ptr(),
            default_string.as_ptr(),
            message.as_mut_ptr() as *mut i8,
            message.len() as i32
        );
    }

    raw_to_cstring(message)
}

fn raw_to_cstring<T: Into<Vec<u8>>>(cs: T) -> CString {
    let mut cs = cs.into();

    if let Some(null_pos) = cs.iter().position(|&c| c == b'\0') {
        cs.resize(null_pos, b'\0');
    }

    CString::new(cs).unwrap()
}

extern "C" fn x11_error_handler(display: *mut xlib::Display, error: *mut xlib::XErrorEvent) -> c_int {
    unsafe {
        let error_message = x11_get_error_message(display, (*error).error_code as i32);
        let request_message = x11_get_request_description(display, (*error).request_code as i32);
        println!(
            "XError: {} (request: {}, resource: {})",
            error_message.to_str().unwrap_or_default(),
            request_message.to_str().unwrap_or_default(),
            (*error).resourceid
        );
    }
    0
}
