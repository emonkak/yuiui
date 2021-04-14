use std::mem;
use std::os::raw::*;
use std::ptr;
use x11::xft;
use x11::xlib;

use context::Context;
use utils;

#[derive(Debug)]
pub struct TrayIcon {
    display: *mut xlib::Display,
    embedder_window: xlib::Window,
    icon_window: xlib::Window,
    status: Status,
    is_selected: bool,
    title: String,
    width: u32,
    height: u32,
}

#[derive(Debug, PartialEq)]
enum Status {
    Initialized,
    Embedded,
    Invalidated,
}

impl TrayIcon {
    pub fn new(context: &mut Context, tray_window: xlib::Window, icon_window: xlib::Window, x: i32, y: i32, width: u32, height: u32) -> Self {
        unsafe {
            let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
            attributes.win_gravity = xlib::NorthWestGravity;
            attributes.background_pixmap = xlib::ParentRelative as u64;

            let embedder_window = xlib::XCreateWindow(
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

            xlib::XSelectInput(
                context.display,
                embedder_window,
                xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::LeaveWindowMask | xlib::EnterWindowMask | xlib::StructureNotifyMask | xlib::PropertyChangeMask | xlib::ExposureMask
            );

            let title = context.get_window_title(icon_window).unwrap_or_default();

            TrayIcon {
                display: context.display,
                embedder_window,
                icon_window,
                status: Status::Initialized,
                is_selected: false,
                title,
                width,
                height,
            }
        }
    }

    pub fn update(&self, context: &mut Context) {
        unsafe {
            let screen_number = xlib::XDefaultScreen(self.display);
            let visual = xlib::XDefaultVisual(self.display, screen_number);
            let colormap = xlib::XDefaultColormap(self.display, screen_number);
            let depth = xlib::XDefaultDepth(self.display, screen_number);

            let pixmap = xlib::XCreatePixmap(self.display, self.embedder_window, self.width, self.height, depth as u32);
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
                (self.height / 2) as i32 - (context.font_set.description().pixel_size / 2) as i32,
                &self.title
            );

            xlib::XCopyArea(self.display, pixmap, self.embedder_window, gc, 0, 0, self.width, self.height, 0, 0);

            xlib::XFreeGC(self.display, gc);
            xlib::XFreePixmap(self.display, pixmap);
            xft::XftDrawDestroy(draw);
        }
    }

    pub fn show(&mut self, context: &Context) {
        println!("show_icon: {:?}", self);

        if self.status == Status::Embedded {
            return;
        }

        self.status = Status::Embedded;
        utils::resize_window(self.display, self.icon_window, context.icon_size, context.icon_size);

        unsafe {
            xlib::XSelectInput(self.display, self.icon_window, xlib::StructureNotifyMask | xlib::PropertyChangeMask | xlib::ExposureMask);
            xlib::XReparentWindow(self.display, self.icon_window, self.embedder_window, 0, 0);
            xlib::XMapRaised(self.display, self.icon_window);
            xlib::XMapWindow(self.display, self.embedder_window);
            xlib::XFlush(self.display);
        }

        println!("show_icon end");
    }

    pub fn wait_for_embedding(&mut self) {
        unsafe {
            xlib::XSelectInput(self.display, self.icon_window, xlib::PropertyChangeMask);
            xlib::XFlush(self.display);
        }
    }

    pub fn invalidate(self) {
        if self.status == Status::Invalidated {
            return;
        }

        let mut self_mut = self;
        self_mut.status = Status::Invalidated;
    }

    pub fn emit_icon_click(&self, button: c_uint, button_mask: c_uint, x: c_int, y: c_int) -> bool {
        println!("emit_click: {} {} {} {}", button, button_mask, x, y);

        let result = utils::emit_crossing_event(
            self.display,
            self.icon_window,
            xlib::EnterNotify,
            xlib::EnterWindowMask,
            xlib::True
        );
        if !result {
            return false;
        }

        let result = utils::emit_button_event(
            self.display,
            self.icon_window,
            xlib::ButtonPress,
            xlib::ButtonPressMask,
            button,
            button_mask,
            x,
            y
        );
        if !result {
            return false;
        }

        let result = utils::emit_button_event(
            self.display,
            self.icon_window,
            xlib::ButtonRelease,
            xlib::ButtonReleaseMask,
            button,
            button_mask,
            x,
            y
        );
        if !result {
            return false;
        }

        let result = utils::emit_crossing_event(
            self.display,
            self.icon_window,
            xlib::LeaveNotify,
            xlib::LeaveWindowMask,
            xlib::False
        );
        if !result {
            return false;
        }

        true
    }

    pub fn emit_icon_press(&self, button: c_uint, button_mask: c_uint, x: c_int, y: c_int) -> bool {
        println!("emit_icon_press: {}", self.icon_window);
        utils::emit_button_event(
            self.display,
            self.icon_window,
            xlib::ButtonPress,
            xlib::ButtonPressMask,
            button,
            button_mask,
            x,
            y
        )
    }

    pub fn emit_icon_release(&self, button: c_uint, button_mask: c_uint, x: c_int, y: c_int) -> bool {
        println!("emit_icon_release: {}", self.icon_window);
        utils::emit_button_event(
            self.display,
            self.icon_window,
            xlib::ButtonRelease,
            xlib::ButtonReleaseMask,
            button,
            button_mask,
            x,
            y
        )
    }

    pub fn emit_icon_enter(&self) -> bool {
        println!("emit_icon_enter: {}", self.icon_window);
        utils::emit_crossing_event(
            self.display,
            self.icon_window,
            xlib::EnterNotify,
            xlib::EnterWindowMask,
            xlib::True
        )
    }

    pub fn emit_icon_leave(&self) -> bool {
        println!("emit_icon_leave: {}", self.icon_window);
        utils::emit_crossing_event(
            self.display,
            self.icon_window,
            xlib::LeaveNotify,
            xlib::LeaveWindowMask,
            xlib::False
        )
    }

    pub fn set_selected(&self, selected: bool) {
        if self.is_selected == selected {
            return;
        }

        if selected {

        } else {

        }
    }

    pub fn embedder_window(&self) -> xlib::Window {
        self.embedder_window
    }

    pub fn icon_window(&self) -> xlib::Window {
        self.icon_window
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        println!("drop_icon: {:?}", self);

        unsafe {
            if self.status == Status::Embedded {
                let screen = xlib::XDefaultScreenOfDisplay(self.display);
                let root = xlib::XRootWindowOfScreen(screen);

                xlib::XSelectInput(self.display, self.icon_window, xlib::NoEventMask);
                xlib::XUnmapWindow(self.display, self.icon_window);
                xlib::XReparentWindow(self.display, self.icon_window, root, 0, 0);
                xlib::XMapRaised(self.display, self.icon_window);
            }

            xlib::XDestroyWindow(self.display, self.embedder_window);
        }
    }
}
