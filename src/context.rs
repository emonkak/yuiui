use nix::sys::epoll;
use nix::sys::signal;
use std::env;
use std::ffi::CString;
use std::mem;
use std::os::raw;
use std::os::unix::io::RawFd;
use std::ptr;
use x11::xft;
use x11::xlib;
use x11::xrender;

use config::Config;
use error_handler;
use font_set::FontSet;
use signal::SignalHandler;

pub struct Context {
    pub display: *mut xlib::Display,
    pub config: Config,
    pub atoms: Atoms,
    pub font_set: FontSet,
    pub border_color: Color,
    pub normal_background: Color,
    pub normal_foreground: Color,
    pub selected_background: Color,
    pub selected_foreground: Color,
    pub old_error_handler: Option<unsafe extern "C" fn (*mut xlib::Display, *mut xlib::XErrorEvent) -> raw::c_int>,
    pub signal_handler: SignalHandler,
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
            config,
            atoms: Atoms::new(display),
            font_set: FontSet::new(&config.font_family)
                .ok_or(format!("Failed to initialize `font_family`: {:?}", config.font_family))?,
            border_color: Color::parse(display, &config.border_color)
                .ok_or(format!("Failed to parse `border_color`: {:?}", config.border_color))?,
            normal_background: Color::parse(display, &config.normal_background)
                .ok_or(format!("Failed to parse `normal_background`: {:?}", config.normal_background))?,
            normal_foreground: Color::parse(display, &config.normal_foreground)
                .ok_or(format!("Failed to parse `normal_foreground`: {:?}", config.normal_foreground))?,
            selected_background: Color::parse(display, &config.selected_background)
                .ok_or(format!("Failed to parse `selected_background`: {:?}", config.selected_background))?,
            selected_foreground: Color::parse(display, &config.selected_foreground)
                .ok_or(format!("Failed to parse `selected_foreground`: {:?}", config.selected_foreground))?,
            old_error_handler,
            signal_handler,
        })
    }

    pub fn poll_events<F>(&self, callback: F) where F: FnMut(Event) -> bool {
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
            let num_fds = epoll::epoll_wait(epoll_fd, &mut epoll_events, -1).unwrap_or(0);

            for i in 0..num_fds {
                let changed_fd = epoll_events[i].data() as RawFd;
                if changed_fd == x11_fd {
                    unsafe {
                        xlib::XNextEvent(self.display, &mut event);
                    }

                    if !callback(Event::XEvent(event)) {
                        break 'outer;
                    }
                } else if changed_fd == signal_fd {
                    if let Ok(signal) = self.signal_handler.try_read() {
                        if !callback(Event::Signal(signal)) {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            xlib::XCloseDisplay(self.display);
            xlib::XSetErrorHandler(self.old_error_handler);
        }
    }
}

struct Color {
    color: xlib::XColor,
}

impl Color {
    fn parse(display: *mut xlib::Display, color_spec: &str) -> Option<Self> {
        let color_spec_cstr = CString::new(color_spec).ok()?;
        let screen_number = xlib::XDefaultScreen(display);
        let colormap = xlib::XDefaultColormap(display, screen_number);
        let mut color: xlib::XColor = mem::MaybeUninit::uninit().assume_init();

        if xlib::XParseColor(display, colormap, color_spec_cstr.as_ptr(), &mut color) != 0 {
            return Some(Self{ color })
        }

        None
    }

    pub fn as_xcolor(&self) -> xlib::XColor {
        self.color
    }

    pub fn as_xft_color(&self, alpha: u16) -> xft::XftColor {
        xft::XftColor {
            color: xrender::XRenderColor {
                red: self.color.red,
                green: self.color.green,
                blue: self.color.blue,
                alpha,
            },
            pixel: self.color.pixel
        }
    }
}

struct Atoms {
    pub Manager: xlib::Atom,
    pub NetSystemTrayMessageData: xlib::Atom,
    pub NetSystemTrayOpcode: xlib::Atom,
    pub WMDeleteWindow: xlib::Atom,
    pub WMPing: xlib::Atom,
    pub WMProtocols: xlib::Atom,
    pub WMTakeFocus: xlib::Atom,
}

impl Atoms {
    fn new(display: *mut xlib::Display) -> Self {
        Self {
            Manager: new_atom(display, "MANAGER\0"),
            NetSystemTrayMessageData: new_atom(display, "_NET_SYSTEM_TRAY_MESSAGE_DATA\0"),
            NetSystemTrayOpcode: new_atom(display, "_NET_SYSTEM_TRAY_OPCODE\0"),
            WMDeleteWindow: new_atom(display, "WM_DELETE_WINDOW\0"),
            WMPing: new_atom(display, "WM_PING\0"),
            WMProtocols: new_atom(display, "MANAGER\0"),
            WMTakeFocus: new_atom(display, "WM_TAKE_FOCUS\0"),
        }
    }
}

fn new_atom(display: *mut xlib::Display, null_terminated_name: &str) -> xlib::Atom {
    unsafe {
        xlib::XInternAtom(display, null_terminated_name.as_ptr() as *const raw::c_char, xlib::False)
    }
}
