use std::cmp;
use std::mem;
use std::os::raw::*;
use x11::keysym;
use x11::xlib;

use context::Context;
use font::FontRenderer;
use icon::TrayIcon;

const SYSTEM_TRAY_REQUEST_DOCK: i64 = 0;
const SYSTEM_TRAY_BEGIN_MESSAGE: i64 = 1;
const SYSTEM_TRAY_CANCEL_MESSAGE: i64 = 2;

pub struct Tray<'a> {
    context: &'a Context,
    font_renderer: FontRenderer,
    window: xlib::Window,
    icons: Vec<TrayIcon<'a>>,
    selected_icon_index: Option<usize>,
}

impl<'a> Tray<'a> {
    pub fn new(context: &Context) -> Tray {
        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(context.display);
            let root = xlib::XRootWindowOfScreen(screen);

            let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
            attributes.backing_store = xlib::WhenMapped;
            attributes.bit_gravity = xlib::NorthWestGravity;
            attributes.win_gravity = xlib::NorthWestGravity;

            let window = xlib::XCreateWindow(
                context.display,
                root,
                0,
                0,
                context.icon_size,
                context.window_width,
                0,
                xlib::CopyFromParent,
                xlib::InputOutput as u32,
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CWBitGravity | xlib::CWWinGravity,
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
                xlib::KeyPressMask | xlib::KeyReleaseMask | xlib::StructureNotifyMask | xlib::FocusChangeMask | xlib::PropertyChangeMask | xlib::ExposureMask
            );

            Tray {
                context,
                font_renderer: FontRenderer::new(),
                window,
                icons: Vec::new(),
                selected_icon_index: None,
            }
        }
    }

    pub fn show(&self) {
        unsafe {
            xlib::XMapWindow(self.context.display, self.window);
            xlib::XFlush(self.context.display);
        }
    }

    pub fn hide(&self) {
        unsafe {
            xlib::XUnmapWindow(self.context.display, self.window);
            xlib::XFlush(self.context.display);
        }
    }

    pub fn update(&self) {
        unsafe {
            xlib::XResizeWindow(
                self.context.display,
                self.window,
                self.context.window_width,
                self.context.icon_size * cmp::max(1, self.icons.len()) as u32
            );
            xlib::XFlush(self.context.display);
        }
    }

    pub fn window(&self) -> xlib::Window {
        self.window
    }

    pub fn on_event(&mut self, event: xlib::XEvent) -> bool {
        match event.get_type() {
            xlib::KeyRelease => self.on_key_release(xlib::XKeyEvent::from(event)),
            xlib::ClientMessage => self.on_client_message(xlib::XClientMessageEvent::from(event)),
            xlib::DestroyNotify => self.on_destroy_notify(xlib::XDestroyWindowEvent::from(event)),
            xlib::Expose => self.on_expose(xlib::XExposeEvent::from(event)),
            xlib::PropertyNotify => self.on_property_notify(xlib::XPropertyEvent::from(event)),
            xlib::ReparentNotify => self.on_reparent_notify(xlib::XReparentEvent::from(event)),
            _ => true,
        }
    }

    fn on_key_release(&mut self, event: xlib::XKeyEvent) -> bool {
        let keysym = unsafe {
            xlib::XkbKeycodeToKeysym(
                self.context.display,
                event.keycode as c_uchar,
                if event.state & xlib::ShiftMask != 0 { 1 } else { 0 },
                0
            )
        };
        match keysym as c_uint {
            keysym::XK_Down | keysym::XK_j => self.select_next_icon(),
            keysym::XK_Up | keysym::XK_k => self.select_previous_icon(),
            keysym::XK_Right | keysym::XK_l => self.click_selected_icon(xlib::Button1, xlib::Button1Mask),
            keysym::XK_Left | keysym::XK_h => self.click_selected_icon(xlib::Button3, xlib::Button3Mask),
            _ => (),
        }
        true
    }

    fn on_client_message(&mut self, event: xlib::XClientMessageEvent) -> bool {
        if event.message_type == self.context.atoms.WM_PROTOCOLS && event.format == 32 {
            let protocol = event.data.get_long(0) as xlib::Atom;
            if protocol == self.context.atoms.WM_DELETE_WINDOW {
                return false;
            }
        } else if event.message_type == self.context.atoms.NET_SYSTEM_TRAY_OPCODE {
            let opcode = event.data.get_long(1);
            if opcode == SYSTEM_TRAY_REQUEST_DOCK {
                let window = event.data.get_long(2) as xlib::Window;
                self.add_icon(window);
                self.update();
            } else if opcode == SYSTEM_TRAY_BEGIN_MESSAGE {
                // TODO:
            } else if opcode == SYSTEM_TRAY_CANCEL_MESSAGE {
                // TODO:
            }
        } else if event.message_type == self.context.atoms.NET_SYSTEM_TRAY_MESSAGE_DATA {
            // TODO:
        }
        true
    }

    fn on_destroy_notify(&mut self, event: xlib::XDestroyWindowEvent) -> bool {
        if event.window == self.window {
            return false;
        }
        if let Some(icon) = self.remove_icon(event.window) {
            icon.invalidate();
        }
        true
    }

    fn on_expose(&mut self, event: xlib::XExposeEvent) -> bool {
        if self.window == event.window {
            self.update();
        }
        true
    }

    fn on_property_notify(&mut self, event: xlib::XPropertyEvent) -> bool {
        if event.atom == self.context.atoms.XEMBED_INFO {
            let unmapped = match self.index_of_embedder_window(event.window) {
                Some(index) => {
                    if let Some(embed_info) = self.context.get_xembed_info(event.window) {
                        if !embed_info.is_mapped() {
                            return true;
                        }
                        let icon = unsafe { self.icons.get_unchecked_mut(index) };
                        icon.show();
                        icon.render(&mut self.font_renderer);
                    }
                    false
                },
                _ => false
            };
            if unmapped {
                self.remove_icon(event.window);
            }
        }
        true
    }

    fn on_reparent_notify(&mut self, event: xlib::XReparentEvent) -> bool {
        if let Some(index) = self.index_of_icon_window(event.window) {
            let icon = unsafe { self.icons.get_unchecked_mut(index) };
            if icon.embedder_window() != event.parent {
                self.remove_icon(event.window);
            }
        }
        true
    }

    fn index_of_icon_window(&mut self, window: xlib::Window) -> Option<usize> {
        self.icons.iter().position(|icon| icon.icon_window() == window)
    }

    fn index_of_embedder_window(&mut self, window: xlib::Window) -> Option<usize> {
        self.icons.iter().position(|icon| icon.embedder_window() == window)
    }

    fn add_icon(&mut self, icon_window: xlib::Window) {
        if self.icons.iter().any(|icon| icon.icon_window() == icon_window) {
            return;
        }

        if let Some(embed_info) = self.context.get_xembed_info(icon_window) {
            let mut icon = TrayIcon::new(
                self.context,
                self.window,
                icon_window,
                0,
                self.icons.len() as i32 * self.context.icon_size as i32,
                self.context.window_width,
                self.context.icon_size
            );

            if embed_info.is_mapped() {
                icon.show();
                icon.render(&mut self.font_renderer);
            } else {
                icon.wait_for_embedding();
            }

            self.context.send_embedded_notify(
                icon_window,
                xlib::CurrentTime,
                icon.embedder_window(),
                embed_info.version
            );

            self.icons.push(icon);
        }
    }

    fn remove_icon(&mut self, icon_window: xlib::Window) -> Option<TrayIcon> {
        self.icons.iter()
            .position(|icon| icon.icon_window() == icon_window)
            .map(|index| {
                let icon = self.icons.remove(index);
                self.update();
                icon
            })
    }

    fn click_selected_icon(&mut self, button: c_uint, button_mask: c_uint) {
        println!("Tray.click_selected_icon({:?}): {:?}", button, self.selected_icon_index);

        match self.selected_icon_index {
            Some(index) => {
                let icon = &self.icons[index];
                icon.emit_icon_click(button, button_mask, 10, 10);
            },
            _ => (),
        }
    }

    fn select_next_icon(&mut self) {
        if self.icons.len() == 0 {
            return;
        }

        let selected_icon_index = match self.selected_icon_index {
            Some(index) if index < self.icons.len() - 1 => index + 1,
            _ => 0,
        };

        println!("Tray.select_next_icon(): {}", selected_icon_index);

        self.update_selected_icon_index(selected_icon_index);
    }

    fn select_previous_icon(&mut self) {
        if self.icons.len() == 0 {
            return;
        }

        let selected_icon_index = match self.selected_icon_index {
            Some(index) if index > 0 => index - 1,
            _ => self.icons.len() - 1,
        };

        println!("Tray.select_previous_icon(): {}", selected_icon_index);

        self.update_selected_icon_index(selected_icon_index);
    }

    fn update_selected_icon_index(&mut self, index: usize) {
        if let Some(prev_selected_icon_index) = self.selected_icon_index {
            let prev_icon = unsafe { self.icons.get_unchecked_mut(prev_selected_icon_index) };
            prev_icon.set_selected(false);
            prev_icon.render(&mut self.font_renderer);
        }

        let icon = unsafe { self.icons.get_unchecked_mut(index) };
        icon.set_selected(true);
        icon.render(&mut self.font_renderer);

        self.selected_icon_index = Some(index);
    }
}

impl<'a> Drop for Tray<'a> {
    fn drop(&mut self) {
        self.icons.clear();

        unsafe {
            xlib::XDestroyWindow(self.context.display, self.window);
        }
    }
}
