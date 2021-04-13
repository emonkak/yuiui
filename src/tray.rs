use std::cmp;
use std::mem;
use x11::xlib;

use context::Context;
use icon::TrayIcon;

const SYSTEM_TRAY_REQUEST_DOCK: i64 = 0;
const SYSTEM_TRAY_BEGIN_MESSAGE: i64 = 1;
const SYSTEM_TRAY_CANCEL_MESSAGE: i64 = 2;

pub struct Tray {
    pub display: *mut xlib::Display,
    pub window: xlib::Window,
    icons: Vec<TrayIcon>,
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

            xlib::XSetWMProtocols(
                context.display,
                window,
                protocol_atoms.as_mut_ptr(),
                protocol_atoms.len() as i32
            );

            xlib::XSelectInput(
                context.display,
                window,
                xlib::StructureNotifyMask | xlib::FocusChangeMask | xlib::PropertyChangeMask | xlib::ExposureMask
            );

            Tray {
                display: context.display,
                window,
                icons: Vec::new(),
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

    pub fn update(&self, context: &mut Context) {
        unsafe {
            xlib::XResizeWindow(
                self.display,
                self.window,
                600,
                context.icon_size * cmp::max(1, self.icons.len()) as u32
            );
            xlib::XFlush(self.display);
        }
    }

    pub fn on_event(&mut self, context: &mut Context, event: xlib::XEvent) -> bool {
        match event.get_type() {
            xlib::ButtonRelease => self.on_button_release(context, xlib::XButtonEvent::from(event)),
            xlib::ClientMessage => self.on_client_message(context, xlib::XClientMessageEvent::from(event)),
            xlib::DestroyNotify => self.on_destroy_notify(context, xlib::XDestroyWindowEvent::from(event)),
            xlib::Expose => self.on_expose(context, xlib::XExposeEvent::from(event)),
            xlib::PropertyNotify => self.on_property_notify(context, xlib::XPropertyEvent::from(event)),
            xlib::ReparentNotify => self.on_reparent_notify(context, xlib::XReparentEvent::from(event)),
            _ => true,
        }
    }

    fn on_button_release(&mut self, _: &mut Context, event: xlib::XButtonEvent) -> bool {
        if let Some(icon) = self.get_icon_by_embedder_window(event.window) {
            icon.emit_click(event.button, event.state, 0, 0);
        }
        true
    }

    fn on_client_message(&mut self, context: &mut Context, event: xlib::XClientMessageEvent) -> bool {
        if event.message_type == context.atoms.WM_PROTOCOLS && event.format == 32 {
            let protocol = event.data.get_long(0) as xlib::Atom;
            if protocol == context.atoms.WM_DELETE_WINDOW {
                return false;
            }
        } else if event.message_type == context.atoms.NET_SYSTEM_TRAY_OPCODE {
            let opcode = event.data.get_long(1);
            if opcode == SYSTEM_TRAY_REQUEST_DOCK {
                let window = event.data.get_long(2) as xlib::Window;
                self.add_icon(context, window);
                self.update(context);
            } else if opcode == SYSTEM_TRAY_BEGIN_MESSAGE {
                //
            } else if opcode == SYSTEM_TRAY_CANCEL_MESSAGE {
                //
            }
        } else if event.message_type == context.atoms.NET_SYSTEM_TRAY_MESSAGE_DATA {
            //
        }
        true
    }

    fn on_destroy_notify(&mut self, context: &mut Context, event: xlib::XDestroyWindowEvent) -> bool {
        if event.window == self.window {
            return false;
        }
        if let Some(icon) = self.remove_icon(context, event.window) {
            icon.invalidate();
        }
        true
    }

    fn on_property_notify(&mut self, context: &mut Context, event: xlib::XPropertyEvent) -> bool {
        if event.atom == context.atoms.XEMBED_INFO {
            let unmapped = match self.get_icon_by_icon_window(event.window) {
                Some(icon) => {
                    if let Some(embed_info) = context.get_xembed_info(event.window) {
                        if !embed_info.is_mapped() {
                            return true;
                        }
                        icon.show(context);
                    }
                    false
                },
                _ => false
            };
            if unmapped {
                self.remove_icon(context, event.window);
            }
        }
        true
    }

    fn on_reparent_notify(&mut self, context: &mut Context, event: xlib::XReparentEvent) -> bool {
        if let Some(icon) = self.get_icon_by_icon_window(event.window) {
            if icon.embedder_window() != event.parent {
                self.remove_icon(context, event.window);
            }
        }
        true
    }

    fn on_expose(&mut self, context: &mut Context, event: xlib::XExposeEvent) -> bool {
        if self.window == event.window {
            self.update(context);
        } else if let Some(icon) = self.get_icon_by_icon_window(event.window) {
            icon.update(context);
        }
        true
    }

    fn get_icon_by_icon_window(&mut self, window: xlib::Window) -> Option<&mut TrayIcon> {
        self.icons.iter()
            .position(|icon| icon.icon_window() == window)
            .map(move |index| unsafe {
                self.icons.get_unchecked_mut(index)
            })
    }

    fn get_icon_by_embedder_window(&mut self, window: xlib::Window) -> Option<&mut TrayIcon> {
        self.icons.iter()
            .position(|icon| icon.embedder_window() == window)
            .map(move |index| unsafe {
                self.icons.get_unchecked_mut(index)
            })
    }

    fn add_icon(&mut self, context: &mut Context, icon_window: xlib::Window) {
        if self.icons.iter().any(|icon| icon.icon_window() == icon_window) {
            return;
        }

        if let Some(embed_info) = context.get_xembed_info(icon_window) {
            let mut icon = TrayIcon::new(
                context,
                self.window,
                icon_window,
                0,
                self.icons.len() as i32 * context.icon_size as i32,
                600,
                context.icon_size
            );

            if embed_info.is_mapped() {
                icon.show(context);
            } else {
                icon.wait_for_embedding();
            }

            context.send_embedded_notify(
                icon_window,
                xlib::CurrentTime,
                icon.embedder_window(),
                embed_info.version
            );

            self.icons.push(icon);
        }
    }

    fn remove_icon(&mut self, context: &mut Context, icon_window: xlib::Window) -> Option<TrayIcon> {
        self.icons.iter()
            .position(|icon| icon.icon_window() == icon_window)
            .map(|index| {
                let icon = self.icons.remove(index);
                self.update(context);
                icon
            })
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        self.icons.clear();

        unsafe {
            xlib::XDestroyWindow(self.display, self.window);
        }
    }
}
