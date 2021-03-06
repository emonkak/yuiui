use atom_store::AtomStore;
use event_handler::EventHandler;
use std::collections::HashMap;
use std::mem;
use std::os::raw::*;
use x11::xlib;

const ICON_SIZE: u32 = 64;

const SYSTEM_TRAY_REQUEST_DOCK: i64 = 0;
const SYSTEM_TRAY_BEGIN_MESSAGE: i64 = 1;
const SYSTEM_TRAY_CANCEL_MESSAGE: i64 = 2;

pub struct Tray {
    display: *mut xlib::Display,
    window: xlib::Window,
    icons: HashMap<xlib::Window, TrayIcon>,
    atom_store: AtomStore,
}

struct TrayIcon {
    display: *mut xlib::Display,
    window: xlib::Window,
    wrapper: xlib::Window,
    status: TrayIconStatus,
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
                display,
                window,
                atom_store,
                icons: HashMap::new(),
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

    fn add_icon(&mut self, window: xlib::Window) {
        println!("add_icon: {}", window);
        if !self.icons.contains_key(&window) {
            let tray_icon = TrayIcon::new(
                self.display,
                self.window,
                window,
                self.icons.len() as i32 * ICON_SIZE as i32,
                0,
                ICON_SIZE
            );

            self.icons.insert(window, tray_icon);
        }
        self.update_window_dimension();
    }

    fn remove_icon(&mut self, window: xlib::Window) -> Option<TrayIcon> {
        let result = self.icons.remove(&window);
        self.update_window_dimension();
        result
    }

    fn update_window_dimension(&self) {
        unsafe {
            xlib::XResizeWindow(
                self.display,
                self.window,
                ICON_SIZE * self.icons.len() as u32,
                ICON_SIZE
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
            if icon.wrapper != event.parent {
                self.remove_icon(event.window);
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
    fn new(display: *mut xlib::Display, tray_window: xlib::Window, window: xlib::Window, x: i32, y: i32, size: u32) -> Self {
        unsafe {
            let mut size_hints: xlib::XSizeHints = mem::MaybeUninit::uninit().assume_init();
            size_hints.flags = xlib::PSize;
            size_hints.width = size as i32;
            size_hints.height = size as i32;

            xlib::XSetWMNormalHints(display, window, &mut size_hints);
            xlib::XResizeWindow(display, window, size, size);

            xlib::XSelectInput(display, window, xlib::StructureNotifyMask | xlib::PropertyChangeMask);

            let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
            attributes.win_gravity = xlib::NorthWestGravity;
            attributes.background_pixmap = xlib::ParentRelative as u64;

            let wrapper = xlib::XCreateWindow(
                display,
                tray_window,
                x,
                y,
                size,
                size,
                0,
                0,
                xlib::CopyFromParent as u32,
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CWBackPixmap | xlib::CWWinGravity,
                &mut attributes
            );

            xlib::XReparentWindow(display, window, wrapper, 0, 0);
            xlib::XMapRaised(display, window);
            xlib::XMapWindow(display, wrapper);

            TrayIcon {
                display,
                window,
                wrapper,
                status: TrayIconStatus::Handled
            }
        }
    }

    fn unhandle(&mut self) {
        self.status = TrayIconStatus::Unhandled;
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        println!("drop_icon: {}", self.window);

        unsafe {
            if let TrayIconStatus::Handled = self.status {
                let screen = xlib::XDefaultScreenOfDisplay(self.display);
                let root = xlib::XRootWindowOfScreen(screen);

                xlib::XSelectInput(self.display, self.window, xlib::NoEventMask);
                xlib::XUnmapWindow(self.display, self.window);
                xlib::XReparentWindow(self.display, self.window, root, 0, 0);
                xlib::XMapRaised(self.display, self.window);
            }

            xlib::XDestroyWindow(self.display, self.wrapper);
        }
    }
}
