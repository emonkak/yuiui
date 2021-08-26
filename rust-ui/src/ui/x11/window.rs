use raw_window_handle::unix::XlibHandle;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::mem::MaybeUninit;
use x11::xlib;

use crate::geometrics::{PhysicalPoint, PhysicalRectangle};
use crate::graphics::Viewport;

#[derive(Clone, Debug)]
pub struct Window {
    display: *mut xlib::Display,
    window: xlib::Window,
    scale_factor: f32,
}

impl Window {
    pub fn new(display: *mut xlib::Display, window: xlib::Window, scale_factor: f32) -> Self {
        Self {
            display,
            window,
            scale_factor,
        }
    }

    pub fn create(display: *mut xlib::Display, viewport: Viewport, point: PhysicalPoint) -> Self {
        let window = unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let root = xlib::XRootWindowOfScreen(screen);

            let mut attributes: xlib::XSetWindowAttributes = MaybeUninit::zeroed().assume_init();
            attributes.background_pixel = xlib::XWhitePixel(display, screen_number);

            let physical_size = viewport.physical_size();

            xlib::XCreateWindow(
                display,
                root,
                point.x as i32,
                point.y as i32,
                physical_size.width,
                physical_size.height,
                0,
                xlib::CopyFromParent,
                xlib::InputOutput as u32,
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CWBackPixel,
                &mut attributes,
            )
        };

        unsafe {
            xlib::XSelectInput(
                display,
                window,
                xlib::ExposureMask
                    | xlib::StructureNotifyMask
                    | xlib::KeyPressMask
                    | xlib::KeyReleaseMask
                    | xlib::ButtonPressMask
                    | xlib::ButtonReleaseMask
                    | xlib::PointerMotionMask,
            );
        }

        Self {
            display,
            window,
            scale_factor: viewport.scale_factor(),
        }
    }
}

impl crate::ui::Window for Window {
    type WindowId = xlib::Window;

    #[inline]
    fn window_id(&self) -> Self::WindowId {
        self.window
    }

    fn get_bounds(&self) -> PhysicalRectangle {
        unsafe {
            let mut attributes_ptr: MaybeUninit<xlib::XWindowAttributes> = MaybeUninit::uninit();
            xlib::XGetWindowAttributes(self.display, self.window, attributes_ptr.as_mut_ptr());
            let attributes = attributes_ptr.assume_init();
            PhysicalRectangle {
                x: attributes.x as _,
                y: attributes.y as _,
                width: attributes.width as _,
                height: attributes.height as _,
            }
        }
    }

    fn get_scale_factor(&self) -> f32 {
        self.scale_factor
    }

    fn invalidate(&self, bounds: PhysicalRectangle) {
        let mut event = xlib::XEvent::from(xlib::XExposeEvent {
            type_: xlib::Expose,
            serial: 0,
            send_event: xlib::True,
            display: self.display,
            window: self.window,
            x: bounds.x as _,
            y: bounds.y as _,
            width: bounds.width as _,
            height: bounds.height as _,
            count: 0,
        });

        unsafe {
            xlib::XSendEvent(
                self.display,
                self.window,
                xlib::True,
                xlib::NoEventMask,
                &mut event,
            );
            xlib::XFlush(self.display);
        }
    }

    fn show(&self) {
        unsafe {
            xlib::XMapWindow(self.display, self.window);
            xlib::XFlush(self.display);
        }
    }
}

unsafe impl HasRawWindowHandle for Window {
    #[inline]
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Xlib(XlibHandle {
            window: self.window,
            display: self.display as *mut _,
            ..XlibHandle::empty()
        })
    }
}
