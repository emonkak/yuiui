use std::cmp;
use std::collections::HashMap;
use std::ffi::CString;
use std::mem;
use std::ptr;
use x11::xft;
use x11::xlib;

use context::Context;

const SYSTEM_TRAY_REQUEST_DOCK: i64 = 0;
const SYSTEM_TRAY_BEGIN_MESSAGE: i64 = 1;
const SYSTEM_TRAY_CANCEL_MESSAGE: i64 = 2;

pub struct Tray {
    pub display: *mut xlib::Display,
    pub window: xlib::Window,
    items: HashMap<xlib::Window, TrayItem>,
}

impl Tray {
    pub fn new(context: &Context) -> Tray {
        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(context.display);
            let root = xlib::XRootWindowOfScreen(screen);

            let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
            attributes.background_pixel = context.normal_background.pixel();
            attributes.border_pixel = context.border_color.pixel();
            attributes.bit_gravity = xlib::NorthWestGravity;
            attributes.win_gravity = xlib::NorthWestGravity;
            attributes.backing_store = xlib::NotUseful;

            let window = xlib::XCreateWindow(
                context.display,
                root,
                0,
                0,
                context.icon_size,
                context.icon_size,
                0,
                0,
                xlib::CopyFromParent as u32,
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CWBackPixel | xlib::CWBorderPixel | xlib::CWBitGravity | xlib::CWWinGravity | xlib::CWBackingStore,
                &mut attributes
            );

            let mut protocol_atoms = [
                context.atoms.WM_DELETE_WINDOW,
                context.atoms.WM_TAKE_FOCUS,
                context.atoms.WM_PING,
            ];

            xlib::XSetWMProtocols(context.display, window, protocol_atoms.as_mut_ptr(), 3);

            xlib::XSelectInput(
                context.display,
                window,
                xlib::StructureNotifyMask | xlib::FocusChangeMask | xlib::PropertyChangeMask | xlib::ExposureMask
            );

            Tray {
                display: context.display,
                window,
                items: HashMap::new(),
            }
        }
    }

    pub fn show(&self) {
        unsafe {
            xlib::XMapWindow(self.display, self.window);
            xlib::XFlush(self.display);
        }
    }

    pub fn hide(&self) {
        unsafe {
            xlib::XUnmapWindow(self.display, self.window);
            xlib::XFlush(self.display);
        }
    }

    pub fn update_window_dimension(&self, context: &mut Context) {
        unsafe {
            xlib::XResizeWindow(
                self.display,
                self.window,
                600,
                context.icon_size * cmp::max(1, self.items.len()) as u32
            );
        }
    }

    pub fn handle_event(&mut self, context: &mut Context, event: xlib::XEvent) -> bool {
        match event.get_type() {
            xlib::ClientMessage => self.handle_client_message(context, xlib::XClientMessageEvent::from(event)),
            xlib::DestroyNotify => self.handle_destroy_notify(context, xlib::XDestroyWindowEvent::from(event)),
            xlib::ReparentNotify => self.handle_reparent_notify(context, xlib::XReparentEvent::from(event)),
            xlib::Expose => self.handle_expose(context, xlib::XExposeEvent::from(event)),
            _ => true,
        }
    }

    fn handle_client_message(&mut self, context: &mut Context, event: xlib::XClientMessageEvent) -> bool {
        let wm_protocols_atom = context.atoms.WM_PROTOCOLS;
        let wm_delete_window_atom = context.atoms.WM_DELETE_WINDOW;
        let net_system_tray_message_data_atom = context.atoms.NET_SYSTEM_TRAY_MESSAGE_DATA;
        let net_system_tray_opcode_atom = context.atoms.NET_SYSTEM_TRAY_OPCODE;

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
                self.add_icon(context, window);
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

    fn handle_destroy_notify(&mut self, context: &mut Context, event: xlib::XDestroyWindowEvent) -> bool {
        if event.window == self.window {
            return false;
        }
        if let Some(mut item) = self.remove_icon(context, event.window) {
            item.mark_as_unhandled();
        }
        true
    }

    fn handle_reparent_notify(&mut self, context: &mut Context, event: xlib::XReparentEvent) -> bool {
        if let Some(item) = self.items.get(&event.window) {
            if item.window != event.parent {
                self.remove_icon(context, event.window);
            }
        }
        true
    }

    fn handle_expose(&mut self, context: &mut Context, event: xlib::XExposeEvent) -> bool {
        if self.window == event.window {
            self.update_window_dimension(context);
        } else {
            for (_, item) in self.items.iter_mut() {
                if item.icon_window == event.window {
                    item.render(context);
                }
            }
        }
        true
    }

    fn add_icon(&mut self, context: &mut Context, icon_window: xlib::Window) {
        println!("NewIcon: {:#02x}", icon_window);

        if !self.items.contains_key(&icon_window) {
            let mut item = TrayItem::new(
                context,
                self.window,
                icon_window,
                0,
                self.items.len() as i32 * context.icon_size as i32,
                600,
                context.icon_size,
                context.icon_size
            );

            item.render(context);

            self.items.insert(icon_window, item);
        }

        self.update_window_dimension(context);
    }

    fn remove_icon(&mut self, context: &mut Context, icon_window: xlib::Window) -> Option<TrayItem> {
        let result = self.items.remove(&icon_window);
        if let Some(_) = result {
            self.update_window_dimension(context);
        }
        result
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        println!("Drop Tray");

        self.items.clear();

        unsafe {
            xlib::XDestroyWindow(self.display, self.window);
        }
    }
}

pub struct TrayItem {
    display: *mut xlib::Display,
    status: TrayItemStatus,
    pub icon_window: xlib::Window,
    pub window: xlib::Window,
    width: u32,
    height: u32,
}

enum TrayItemStatus {
    Handled,
    Unhandled,
}

impl TrayItem {
    pub fn new(context: &Context, tray_window: xlib::Window, icon_window: xlib::Window, x: i32, y: i32, width: u32, height: u32, icon_size: u32) -> Self {
        unsafe {
            let mut size_hints: xlib::XSizeHints = mem::MaybeUninit::uninit().assume_init();
            size_hints.flags = xlib::PSize;
            size_hints.width = icon_size as i32;
            size_hints.height = icon_size as i32;

            xlib::XSetWMNormalHints(context.display, icon_window, &mut size_hints);
            xlib::XResizeWindow(context.display, icon_window, icon_size, icon_size);

            xlib::XSelectInput(context.display, icon_window, xlib::StructureNotifyMask | xlib::PropertyChangeMask | xlib::ExposureMask);

            let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
            attributes.win_gravity = xlib::NorthWestGravity;
            attributes.background_pixmap = xlib::ParentRelative as u64;
            // attributes.background_pixel = context.normal_background.pixel();

            let window = xlib::XCreateWindow(
                context.display,
                tray_window,
                x,
                y,
                width,
                height,
                0,
                0,
                xlib::CopyFromParent as u32,
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CWBackPixmap | xlib::CWWinGravity,
                &mut attributes
            );

            xlib::XReparentWindow(context.display, icon_window, window, 0, 0);
            xlib::XMapRaised(context.display, icon_window);
            xlib::XMapWindow(context.display, window);

            TrayItem {
                display: context.display,
                status: TrayItemStatus::Handled,
                icon_window,
                window,
                width,
                height,
            }
        }
    }

    pub fn render(&mut self, context: &mut Context) {
        let title = unsafe {
            let mut name_ptr: *mut i8 = mem::MaybeUninit::uninit().assume_init();
            let result = xlib::XFetchName(self.display, self.icon_window, &mut name_ptr);
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
            let draw = xft::XftDrawCreate(self.display, pixmap, visual, colormap);
            let gc = xlib::XCreateGC(self.display, pixmap, 0, ptr::null_mut());
            let bg_pixel = context.normal_background.pixel();

            xlib::XSetForeground(self.display, gc, bg_pixel);
            xlib::XFillRectangle(self.display, pixmap, gc, 0, 0, self.width, self.height);


            context.font_renderer.render_line_text(
                self.display,
                draw,
                &mut context.normal_foreground.xft_color(),
                &context.font_set,
                context.icon_size as i32,
                0,
                &title
            );

            xlib::XCopyArea(self.display, pixmap, self.window, gc, 0, 0, self.width, self.height, 0, 0);

            xlib::XFreeGC(self.display, gc);
            xlib::XFreePixmap(self.display, pixmap);
            xft::XftDrawDestroy(draw);

            xlib::XResizeWindow(self.display, self.icon_window, context.icon_size, context.icon_size);
        }
    }

    pub fn mark_as_unhandled(&mut self) {
        self.status = TrayItemStatus::Unhandled;
    }
}

impl Drop for TrayItem {
    fn drop(&mut self) {
        println!("Drop TrayItem: {:#02x}", self.icon_window);

        unsafe {
            if let TrayItemStatus::Handled = self.status {
                let screen = xlib::XDefaultScreenOfDisplay(self.display);
                let root = xlib::XRootWindowOfScreen(screen);

                xlib::XSelectInput(self.display, self.icon_window, xlib::NoEventMask);
                xlib::XUnmapWindow(self.display, self.icon_window);
                xlib::XReparentWindow(self.display, self.icon_window, root, 0, 0);
                xlib::XMapRaised(self.display, self.icon_window);
            }

            xlib::XDestroyWindow(self.display, self.window);
        }
    }
}
