use nix::sys::epoll;
use nix::sys::signal;
use std::borrow::Borrow;
use std::env;
use std::ffi::CString;
use std::mem;
use std::os::raw::*;
use std::os::unix::io::RawFd;
use std::ptr;
use x11::xft;
use x11::xlib;
use x11::xrender;

use config::Config;
use error_handler;
use font::FontDescription;
use font::FontRenderer;
use font::FontSet;
use signal_handler::SignalHandler;
use utils;
use xembed::XEmbedInfo;
use xembed::XEmbedMessage;

pub struct Context {
    pub display: *mut xlib::Display,
    pub atoms: Atoms,
    pub icon_size: u32,
    pub font_set: FontSet,
    pub font_renderer: FontRenderer,
    pub border_color: Color,
    pub normal_background: Color,
    pub normal_foreground: Color,
    pub selected_background: Color,
    pub selected_foreground: Color,
    signal_handler: SignalHandler,
    old_error_handler: Option<unsafe extern "C" fn (*mut xlib::Display, *mut xlib::XErrorEvent) -> c_int>,
}

pub enum Event {
    XEvent(xlib::XEvent),
    Signal(signal::Signal),
}

impl Context {
    pub fn new(config: Config) -> Result<Self, String> {
        let signal_handler = SignalHandler::install(signal::Signal::SIGINT)
            .map_err(|error| error.to_string())?;

        let old_error_handler = unsafe {
            xlib::XSetErrorHandler(Some(error_handler::handle))
        };

        let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
        if display.is_null() {
            return Err(format!(
                    "No display found at {}",
                    env::var("DISPLAY").unwrap_or_default()
                )
            );
        }

        Ok(Context {
            display,
            atoms: Atoms::new(display),
            icon_size: config.icon_size,
            font_set: FontSet::new(FontDescription {
                    family_name: config.font_family.clone(),
                    weight: config.font_weight,
                    style: config.font_style,
                    pixel_size: config.font_size,
                })
                .ok_or(format!("Failed to initialize `font_set`: {:?}", config.font_family))?,
            font_renderer: FontRenderer::new(),
            border_color: Color::new(display, &config.border_color)
                .ok_or(format!("Failed to parse `border_color`: {:?}", config.border_color))?,
            normal_background: Color::new(display, &config.normal_background)
                .ok_or(format!("Failed to parse `normal_background`: {:?}", config.normal_background))?,
            normal_foreground: Color::new(display, &config.normal_foreground)
                .ok_or(format!("Failed to parse `normal_foreground`: {:?}", config.normal_foreground))?,
            selected_background: Color::new(display, &config.selected_background)
                .ok_or(format!("Failed to parse `selected_background`: {:?}", config.selected_background))?,
            selected_foreground: Color::new(display, &config.selected_foreground)
                .ok_or(format!("Failed to parse `selected_foreground`: {:?}", config.selected_foreground))?,
            signal_handler,
            old_error_handler,
        })
    }

    pub fn poll_events<F>(&mut self, mut callback: F) where F: FnMut(&mut Context, Event) -> bool {
        let epoll_fd = epoll::epoll_create().unwrap();

        let x11_fd = unsafe { xlib::XConnectionNumber(self.display) as RawFd };
        let mut x11_ev = epoll::EpollEvent::new(epoll::EpollFlags::EPOLLIN, x11_fd as u64);
        epoll::epoll_ctl(epoll_fd, epoll::EpollOp::EpollCtlAdd, x11_fd, Some(&mut x11_ev)).unwrap();

        let signal_fd = self.signal_handler.fd();
        let mut signal_ev = epoll::EpollEvent::new(epoll::EpollFlags::EPOLLIN, signal_fd as u64);
        epoll::epoll_ctl(epoll_fd, epoll::EpollOp::EpollCtlAdd, signal_fd, Some(&mut signal_ev)).unwrap();

        let mut epoll_events = [epoll::EpollEvent::empty(); 2];
        let mut event: xlib::XEvent = unsafe { mem::MaybeUninit::uninit().assume_init() };

        'outer: loop {
            let changed_fds = epoll::epoll_wait(epoll_fd, &mut epoll_events, -1).unwrap_or(0);

            for i in 0..changed_fds {
                let fd = epoll_events[i].data() as RawFd;
                if fd == x11_fd {
                    let pendings = unsafe { xlib::XPending(self.display) };
                    for _ in 0..pendings {
                        unsafe {
                            xlib::XNextEvent(self.display, &mut event);
                        }

                        if !callback(self, Event::XEvent(event)) {
                            break 'outer;
                        }
                    }
                } else if fd == signal_fd {
                    if let Ok(signal) = self.signal_handler.try_read() {
                        if !callback(self, Event::Signal(signal)) {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    pub fn get_atom<T: Borrow<str>>(&self, null_terminated_name: T) -> xlib::Atom {
        utils::new_atom(self.display, null_terminated_name.borrow())
    }

    pub fn get_xembed_info(&self, window: xlib::Window) -> Option<XEmbedInfo> {
        utils::get_window_property::<_, 2>(self.display, window, self.atoms.XEMBED_INFO).map(|prop| {
            XEmbedInfo {
                version: (*prop)[0],
                flags: (*prop)[1],
            }
        })
    }

    pub fn get_window_title(&self, window: xlib::Window) -> Option<String> {
        let mut name_ptr: *mut i8 = ptr::null_mut();

        let result = unsafe { xlib::XFetchName(self.display, window, &mut name_ptr) };
        if result == xlib::True && !name_ptr.is_null() && unsafe { *name_ptr } != 0 {
            unsafe {
                CString::from_raw(name_ptr).into_string().ok()
            }
        } else {
            utils::get_window_property::<c_ulong, 1>(self.display, window, self.atoms.NET_WM_PID)
                .and_then(|prop| {
                    let pid = prop[0] as u32;
                    utils::get_process_name(pid as u32).ok()
                })
        }
    }

    pub fn send_embedded_notify(&self, window: xlib::Window, timestamp: xlib::Time, embedder_window: xlib::Window, version: u64) {
        let mut data = xlib::ClientMessageData::new();
        data.set_long(0, timestamp as c_long);
        data.set_long(1, XEmbedMessage::EmbeddedNotify as c_long);
        data.set_long(2, embedder_window as c_long);
        data.set_long(3, version as c_long);

        utils::send_client_message(
            self.display,
            window,
            window,
            self.atoms.XEMBED,
            data
        );
    }

    pub fn acquire_tray_selection(&self, tray_window: xlib::Window) -> xlib::Window {
        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(self.display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let root = xlib::XRootWindowOfScreen(screen);
            let net_system_tray_atom = self.get_atom(format!("_NET_SYSTEM_TRAY_S{}\0", screen_number));

            let previous_selection_owner = xlib::XGetSelectionOwner(self.display, net_system_tray_atom);
            xlib::XSetSelectionOwner(self.display, net_system_tray_atom, tray_window, xlib::CurrentTime);

            let mut data = xlib::ClientMessageData::new();
            data.set_long(0, xlib::CurrentTime as c_long);
            data.set_long(1, net_system_tray_atom as c_long);
            data.set_long(2, tray_window as c_long);

            utils::send_client_message(
                self.display,
                root,
                root,
                self.atoms.MANAGER,
                data
            );

            previous_selection_owner
        }
    }

    pub fn release_tray_selection(&self, previous_selection_owner: xlib::Window) {
        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(self.display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let net_system_tray_atom = self.get_atom(format!("_NET_SYSTEM_TRAY_S{}\0", screen_number));

            xlib::XSetSelectionOwner(self.display, net_system_tray_atom, previous_selection_owner, xlib::CurrentTime);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            xlib::XSync(self.display, 0);
            xlib::XCloseDisplay(self.display);
            xlib::XSetErrorHandler(self.old_error_handler);
        }
    }
}

pub struct Color {
    color: xlib::XColor,
}

impl Color {
    pub fn new(display: *mut xlib::Display, color_spec: &str) -> Option<Self> {
        let color_spec_cstr = CString::new(color_spec).ok()?;
        unsafe {
            let screen_number = xlib::XDefaultScreen(display);
            let colormap = xlib::XDefaultColormap(display, screen_number);
            let mut color: xlib::XColor = mem::MaybeUninit::uninit().assume_init();

            if xlib::XParseColor(display, colormap, color_spec_cstr.as_ptr(), &mut color) != 0 {
                return Some(Self { color })
            }
        }
        None
    }

    pub fn pixel(&self) -> c_ulong {
        self.color.pixel
    }

    pub fn xcolor(&self) -> xlib::XColor {
        self.color
    }

    pub fn xft_color(&self) -> xft::XftColor {
        xft::XftColor {
            color: xrender::XRenderColor {
                red: self.color.red,
                green: self.color.green,
                blue: self.color.blue,
                alpha: 0xffff,
            },
            pixel: self.color.pixel
        }
    }
}

#[allow(non_snake_case)]
pub struct Atoms {
    pub MANAGER: xlib::Atom,
    pub NET_SYSTEM_TRAY_MESSAGE_DATA: xlib::Atom,
    pub NET_SYSTEM_TRAY_OPCODE: xlib::Atom,
    pub NET_WM_PID: xlib::Atom,
    pub WM_DELETE_WINDOW: xlib::Atom,
    pub WM_PING: xlib::Atom,
    pub WM_PROTOCOLS: xlib::Atom,
    pub WM_TAKE_FOCUS: xlib::Atom,
    pub XEMBED: xlib:: Atom,
    pub XEMBED_INFO: xlib:: Atom,
}

impl Atoms {
    fn new(display: *mut xlib::Display) -> Self {
        Self {
            MANAGER: utils::new_atom(display, "MANAGER\0"),
            NET_SYSTEM_TRAY_MESSAGE_DATA: utils::new_atom(display, "_NET_SYSTEM_TRAY_MESSAGE_DATA\0"),
            NET_SYSTEM_TRAY_OPCODE: utils::new_atom(display, "_NET_SYSTEM_TRAY_OPCODE\0"),
            NET_WM_PID: utils::new_atom(display, "_NET_WM_PID\0"),
            WM_DELETE_WINDOW: utils::new_atom(display, "WM_DELETE_WINDOW\0"),
            WM_PING: utils::new_atom(display, "WM_PING\0"),
            WM_PROTOCOLS: utils::new_atom(display, "WM_PROTOCOLS\0"),
            WM_TAKE_FOCUS: utils::new_atom(display, "WM_TAKE_FOCUS\0"),
            XEMBED: utils::new_atom(display, "_XEMBED\0"),
            XEMBED_INFO: utils::new_atom(display, "_XEMBED_INFO\0"),
        }
    }
}
