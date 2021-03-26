use std::cmp;
use std::collections::HashMap;
use std::ffi::CString;
use std::mem;
use std::os::raw::*;
use std::ptr;
use x11::xft;
use x11::xlib;
use x11::xrender;

use atom_store::AtomStore;
use event_handler::EventHandler;
use font_set::FontSet;

const ICON_SIZE: u32 = 64;

const SYSTEM_TRAY_REQUEST_DOCK: i64 = 0;
const SYSTEM_TRAY_BEGIN_MESSAGE: i64 = 1;
const SYSTEM_TRAY_CANCEL_MESSAGE: i64 = 2;

pub struct Tray {
    atom_store: AtomStore,
    display: *mut xlib::Display,
    icons: HashMap<xlib::Window, TrayIcon>,
    window: xlib::Window,
}

pub struct TrayIcon {
    display: *mut xlib::Display,
    status: TrayIconStatus,
    text_color: xft::XftColor,
    icon: xlib::Window,
    window: xlib::Window,
    width: u32,
    height: u32,
}

enum TrayIconStatus {
    Handled,
    Unhandled,
}

impl Tray {
    pub fn new(display: *mut xlib::Display) -> Tray {
        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let root = xlib::XRootWindowOfScreen(screen);
            let background_pixel = xlib::XWhitePixel(display, screen_number);

            let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
            attributes.background_pixel = background_pixel;
            attributes.border_pixel = background_pixel;
            attributes.bit_gravity = xlib::NorthWestGravity;
            attributes.win_gravity = xlib::NorthWestGravity;
            attributes.backing_store = xlib::NotUseful;

            let window = xlib::XCreateWindow(
                display,
                root,
                0,
                0,
                ICON_SIZE,
                ICON_SIZE,
                0,
                0,
                xlib::CopyFromParent as u32,
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CWBackPixel | xlib::CWBorderPixel | xlib::CWBitGravity | xlib::CWWinGravity | xlib::CWBackingStore,
                &mut attributes
            );

            let mut atom_store = AtomStore::new(display);
            let mut protocol_atoms = [
                atom_store.get("WM_DELETE_WINDOW"),
                atom_store.get("WM_TAKE_FOCUS"),
                atom_store.get("WM_PING"),
            ];

            xlib::XSetWMProtocols(display, window, protocol_atoms.as_mut_ptr(), 3);

            xlib::XSelectInput(
                display,
                window,
                xlib::StructureNotifyMask | xlib::FocusChangeMask | xlib::PropertyChangeMask | xlib::ExposureMask
            );

            Tray {
                atom_store,
                display,
                icons: HashMap::new(),
                window,
            }
        }
    }

    pub fn acquire_tray_selection(&mut self) {
        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(self.display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let root = xlib::XRootWindowOfScreen(screen);

            let net_system_tray_atom = self.atom_store.get(format!("_NET_SYSTEM_TRAY_S{}", screen_number));
            let manager_atom = self.atom_store.get("MANAGER");

            xlib::XSetSelectionOwner(self.display, net_system_tray_atom, self.window, xlib::CurrentTime);

            let mut client_message_data = xlib::ClientMessageData::new();
            client_message_data.set_long(0, xlib::CurrentTime as c_long);
            client_message_data.set_long(1, net_system_tray_atom as c_long);
            client_message_data.set_long(2, self.window as c_long);

            let mut client_message_event = xlib::XEvent::from(xlib::XClientMessageEvent {
                type_: xlib::ClientMessage,
                serial: 0,
                send_event: xlib::True,
                display: self.display,
                window: root,
                message_type: manager_atom,
                format: 32,
                data: client_message_data,
            });

            xlib::XSendEvent(self.display, root, xlib::False, 0xffffff, &mut client_message_event);
        }
    }

    pub fn show(&self) {
        unsafe {
            xlib::XMapWindow(self.display, self.window);
        }
    }

    fn add_icon(&mut self, icon_window: xlib::Window) {
        println!("add_icon: {}", icon_window);

        if !self.icons.contains_key(&icon_window) {
            let mut icon = TrayIcon::new(
                self,
                icon_window,
                0,
                self.icons.len() as i32 * ICON_SIZE as i32,
                600,
                ICON_SIZE,
                ICON_SIZE
            );

            icon.render(&mut self.font_set, ICON_SIZE);

            self.icons.insert(icon_window, icon);
        }
        self.update_window_dimension();
    }

    fn remove_icon(&mut self, icon: xlib::Window) -> Option<TrayIcon> {
        let result = self.icons.remove(&icon);
        self.update_window_dimension();
        result
    }

    fn update_window_dimension(&self) {
        unsafe {
            xlib::XResizeWindow(
                self.display,
                self.window,
                600,
                ICON_SIZE * cmp::max(1, self.icons.len()) as u32
            );
        }
    }
}

impl EventHandler for Tray {
    fn handle_client_message(&mut self, event: xlib::XClientMessageEvent) -> bool {
        let wm_protocols_atom = self.atom_store.get("WM_PROTOCOLS");
        let wm_delete_window_atom = self.atom_store.get("WM_DELETE_WINDOW");
        let net_system_tray_message_data_atom = self.atom_store.get("_NET_SYSTEM_TRAY_MESSAGE_DATA");
        let net_system_tray_opcode_atom = self.atom_store.get("_NET_SYSTEM_TRAY_OPCODE");

        let message_type = event.message_type;
        if message_type == wm_protocols_atom && event.format == 32 {
            let protocol = event.data.get_long(0) as xlib::Atom;
            if protocol == wm_delete_window_atom {
                return false;
            }
        } else if message_type == net_system_tray_opcode_atom {
            let opcode = event.data.get_long(1);
            if opcode == SYSTEM_TRAY_REQUEST_DOCK {
                let window = event.data.get_long(2) as xlib::Window;
                self.add_icon(window);
            } else if opcode == SYSTEM_TRAY_BEGIN_MESSAGE {
                //
            } else if opcode == SYSTEM_TRAY_CANCEL_MESSAGE {
                //
            }
        } else if message_type == net_system_tray_message_data_atom {
            //
        }
        true
    }

    fn handle_destroy_notify(&mut self, event: xlib::XDestroyWindowEvent) -> bool {
        if event.window == self.window {
            return false;
        }
        if let Some(mut icon) = self.remove_icon(event.window) {
            icon.unhandle();
        }
        true
    }

    fn handle_reparent_notify(&mut self, event: xlib::XReparentEvent) -> bool {
        if let Some(icon) = self.icons.get(&event.window) {
            if icon.window != event.parent {
                self.remove_icon(event.window);
            }
        }
        true
    }

    fn handle_expose(&mut self, event: xlib::XExposeEvent) -> bool {
        if self.window == event.window {
            self.update_window_dimension();
        } else {
            for (_, icon) in self.icons.iter_mut() {
                if icon.icon == event.window {
                    icon.render(&mut self.font_set, ICON_SIZE);
                }
            }
        }
        true
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        self.icons.clear();

        unsafe {
            xlib::XDestroyWindow(self.display, self.window);

            let screen = xlib::XDefaultScreenOfDisplay(self.display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let net_system_tray_atom = self.atom_store.get(format!("_NET_SYSTEM_TRAY_S{}", screen_number));

            xlib::XSetSelectionOwner(self.display, net_system_tray_atom, 0, xlib::CurrentTime);
        }
    }
}

impl TrayIcon {
    fn new(tray: &Tray, icon: xlib::Window, x: i32, y: i32, width: u32, height: u32, icon_size: u32) -> Self {
        unsafe {
            let mut size_hints: xlib::XSizeHints = mem::MaybeUninit::uninit().assume_init();
            size_hints.flags = xlib::PSize;
            size_hints.width = icon_size as i32;
            size_hints.height = icon_size as i32;

            xlib::XSetWMNormalHints(tray.display, icon, &mut size_hints);
            xlib::XResizeWindow(tray.display, icon, icon_size, icon_size);

            xlib::XSelectInput(tray.display, icon, xlib::StructureNotifyMask | xlib::PropertyChangeMask | xlib::ExposureMask);

            let screen = xlib::XDefaultScreenOfDisplay(tray.display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let visual = xlib::XDefaultVisual(tray.display, screen_number);
            let colormap = xlib::XDefaultColormap(tray.display, screen_number);

            let mut color: xlib::XColor = mem::MaybeUninit::uninit().assume_init();
            color.red = 0x8000;
            color.blue = 0x8000;
            color.green = 0x8000;
            xlib::XAllocColor(tray.display, colormap, &mut color);

            let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
            attributes.win_gravity = xlib::NorthWestGravity;
            attributes.background_pixmap = xlib::ParentRelative as u64;
            attributes.background_pixel = color.pixel;

            let window = xlib::XCreateWindow(
                tray.display,
                tray.window,
                x,
                y,
                width,
                height,
                0,
                0,
                xlib::CopyFromParent as u32,
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CWBackPixmap | xlib::CWWinGravity,
                // xlib::CWBackPixel | xlib::CWWinGravity,
                &mut attributes
            );

            xlib::XReparentWindow(tray.display, icon, window, 0, 0);
            xlib::XMapRaised(tray.display, icon);
            xlib::XMapWindow(tray.display, window);

            let mut text_color: xft::XftColor = mem::MaybeUninit::uninit().assume_init();

            let render_color = xrender::XRenderColor {
                red: 0x0000,
                green: 0x0000,
                blue: 0x0000,
                alpha: 0xffff,
            };
            xft::XftColorAllocValue(
                tray.display,
                visual,
                colormap,
                &render_color,
                &mut text_color
            );

            TrayIcon {
                display: tray.display,
                status: TrayIconStatus::Handled,
                text_color,
                icon,
                window,
                width,
                height,
            }
        }
    }

    fn render(&mut self, font_set: &mut FontSet, icon_size: u32) {
        let title = unsafe {
            let mut name_ptr: *mut i8 = mem::MaybeUninit::uninit().assume_init();
            let result = xlib::XFetchName(self.display, self.icon, &mut name_ptr);
            if result == 0 || name_ptr.is_null() {
                "NO NAME".to_string()
            } else {
                CString::from_raw(name_ptr).into_string().unwrap_or_default()
            }
        };

        unsafe {
            let screen_number = xlib::XDefaultScreen(self.display);
            let visual = xlib::XDefaultVisual(self.display, screen_number);
            let colormap = xlib::XDefaultColormap(self.display, screen_number);
            let depth = xlib::XDefaultDepth(self.display, screen_number);

            let pixmap = xlib::XCreatePixmap(self.display, self.window, self.width, self.height, depth as u32);
            let gc = xlib::XCreateGC(self.display, pixmap, 0, ptr::null_mut());
            let draw = xft::XftDrawCreate(self.display, pixmap, visual, colormap);

            let bg_pixel = xlib::XWhitePixel(self.display, screen_number);

            xlib::XSetForeground(self.display, gc, bg_pixel);
            xlib::XFillRectangle(self.display, pixmap, gc, 0, 0, self.width, self.height);

            font_set.render_line_text(
                self.display,
                draw,
                &mut self.text_color,
                32.0,
                ICON_SIZE as i32,
                0,
                &title
            );

            xlib::XCopyArea(self.display, pixmap, self.window, gc, 0, 0, self.width, self.height, 0, 0);

            xlib::XFreeGC(self.display, gc);
            xlib::XFreePixmap(self.display, pixmap);
            xft::XftDrawDestroy(draw);

            xlib::XResizeWindow(self.display, self.icon, icon_size, icon_size);
        }
    }

    fn unhandle(&mut self) {
        self.status = TrayIconStatus::Unhandled;
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        println!("drop_icon: {}", self.icon);

        unsafe {
            if let TrayIconStatus::Handled = self.status {
                let screen = xlib::XDefaultScreenOfDisplay(self.display);
                let root = xlib::XRootWindowOfScreen(screen);

                xlib::XSelectInput(self.display, self.icon, xlib::NoEventMask);
                xlib::XUnmapWindow(self.display, self.icon);
                xlib::XReparentWindow(self.display, self.icon, root, 0, 0);
                xlib::XMapRaised(self.display, self.icon);
            }

            xlib::XDestroyWindow(self.display, self.window);
        }
    }
}
