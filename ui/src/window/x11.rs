use std::env;
use std::ptr;
use std::mem;
use x11::xlib;

use geometrics::{Size, Rectangle};
use paint::{PaintContext, Painter};
use window::{WindowHandle, WindowHandler, WindowProcedure};

pub struct XWindowProcedure {
    pub handler: Box<dyn WindowHandler<XWindowHandle>>,
    pub handle: XWindowHandle,
}

#[derive(Clone)]
pub struct XWindowHandle {
    pub display: *mut xlib::Display,
    pub window: xlib::Window
}

pub struct XPainter {
    display: *mut xlib::Display,
    pixmap: xlib::Pixmap,
    gc: xlib::GC,
}

#[derive(Debug)]
pub enum XEvent {
    MotionNotify(xlib::XMotionEvent),
    ButtonPress(xlib::XButtonEvent),
    ButtonRelease(xlib::XButtonEvent),
    ColormapNotify(xlib::XColormapEvent),
    EnterNotify(xlib::XCrossingEvent),
    LeaveNotify(xlib::XCrossingEvent),
    Expose(xlib::XExposeEvent),
    GraphicsExpose(xlib::XGraphicsExposeEvent),
    NoExpose(xlib::XNoExposeEvent),
    FocusIn(xlib::XFocusChangeEvent),
    FocusOut(xlib::XFocusChangeEvent),
    KeymapNotify(xlib::XKeymapEvent),
    KeyPress(xlib::XKeyEvent),
    KeyRelease(xlib::XKeyEvent),
    PropertyNotify(xlib::XPropertyEvent),
    ResizeRequest(xlib::XResizeRequestEvent),
    CirculateNotify(xlib::XCirculateEvent),
    ConfigureNotify(xlib::XConfigureEvent),
    DestroyNotify(xlib::XDestroyWindowEvent),
    GravityNotify(xlib::XGravityEvent),
    MapNotify(xlib::XMapEvent),
    ReparentNotify(xlib::XReparentEvent),
    UnmapNotify(xlib::XUnmapEvent),
    CreateNotify(xlib::XCreateWindowEvent),
    CirculateRequest(xlib::XCirculateRequestEvent),
    ConfigureRequest(xlib::XConfigureRequestEvent),
    MapRequest(xlib::XMapRequestEvent),
    ClientMessage(xlib::XClientMessageEvent),
    MappingNotify(xlib::XMappingEvent),
    SelectionClear(xlib::XSelectionClearEvent),
    SelectionNotify(xlib::XSelectionEvent),
    SelectionRequest(xlib::XSelectionRequestEvent),
    VisibilityNotify(xlib::XVisibilityEvent),
    Any(xlib::XAnyEvent),
}

impl XWindowProcedure {
    pub fn dispatch_event(&self, event: &xlib::XEvent) -> bool {
        if unsafe { event.any.window } == self.handle.window {
            self.handle_event(&event.into())
        } else {
            true
        }
    }

    fn render(&self) {
        let size = self.handle.get_size();
        let pixmap = unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(self.handle.display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let depth = xlib::XDefaultDepth(self.handle.display, screen_number);
            xlib::XCreatePixmap(
                self.handle.display,
                self.handle.window,
                size.width as _,
                size.height as _,
                depth as _
            )
        };
        let gc = unsafe {
            xlib::XCreateGC(self.handle.display, pixmap, 0, ptr::null_mut())
        };

        let mut paint_context = PaintContext::new(XPainter {
            display: self.handle.display,
            pixmap,
            gc
        });

        self.handler.paint(&mut paint_context);
    }
}

impl WindowProcedure<XWindowHandle, XEvent> for XWindowProcedure {
    fn connect(&self, handle: &XWindowHandle) {
        self.handler.connect(handle);
    }

    fn handle_event(&self, event: &XEvent) -> bool {
        println!("{:?}", event);
        match event {
            XEvent::Expose(_) => {
                self.render();
            },
            _ => (),
        }
        true
    }
}

impl XWindowHandle {
    pub fn new(width: u32, height: u32) -> Result<XWindowHandle, String> {
        let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
        if display.is_null() {
            return Err(format!(
                    "No display found at {}",
                    env::var("DISPLAY").unwrap_or_default()
                )
            );
        }

        let window = unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let root = xlib::XRootWindowOfScreen(screen);

            let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
            attributes.background_pixel = xlib::XWhitePixel(display, screen_number);

            xlib::XCreateWindow(
                display,
                root,
                0,
                0,
                width,
                height,
                0,
                xlib::CopyFromParent,
                xlib::InputOutput as u32,
                xlib::CopyFromParent as *mut xlib::Visual,
                xlib::CWBackPixel,
                &mut attributes
            )
        };

        unsafe {
            xlib::XSelectInput(
                display,
                window,
                xlib::ExposureMask
            );
        }

        Ok(XWindowHandle {
            display,
            window,
        })
    }
}

impl WindowHandle for XWindowHandle {
    fn show(&self) {
        unsafe {
            xlib::XMapWindow(self.display, self.window);
            xlib::XFlush(self.display);
        }
    }

    fn close(&self) {
        unsafe {
            xlib::XDestroyWindow(self.display, self.window);
        }
    }

    fn get_size(&self) -> Size {
        let mut attributes: xlib::XWindowAttributes = unsafe { mem::MaybeUninit::zeroed().assume_init() };
        unsafe {
            xlib::XGetWindowAttributes(
                self.display,
                self.window,
                &mut attributes
            );
        }
        Size {
            width: attributes.width as _,
            height: attributes.height as _
        }
    }
}

impl XPainter {
    fn alloc_color(&self, rgba: u32) -> xlib::XColor {
        let mut color = xlib::XColor {
            pixel: 0,
            red: (((rgba & 0xff000000) >> 24) * 0x101) as u16,
            green: (((rgba & 0x00ff0000) >> 16) * 0x101) as u16,
            blue: (((rgba & 0x0000ff00) >> 8) * 0x101) as u16,
            flags: 0,
            pad: 0,
        };

        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(self.display);
            let screen_number = xlib::XScreenNumberOfScreen(screen);
            let colormap = xlib::XDefaultColormap(self.display, screen_number);
            xlib::XAllocColor(self.display, colormap, &mut color);
        };

        color
    }
}

impl Painter<XWindowHandle> for XPainter  {
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle) {
        unsafe {
            let color = self.alloc_color(color);
            xlib::XSetForeground(self.display, self.gc, color.pixel);
            xlib::XFillRectangle(
                self.display,
                self.pixmap,
                self.gc,
                rectangle.point.x as _,
                rectangle.point.y as _,
                rectangle.size.width as _,
                rectangle.size.height as _
            );
        }
    }

    fn commit(&mut self, window: &XWindowHandle, rectangle: &Rectangle) {
        unsafe {
            xlib::XCopyArea(
                self.display,
                self.pixmap,
                window.window,
                self.gc,
                rectangle.point.x as _,
                rectangle.point.y as _,
                rectangle.size.width as _,
                rectangle.size.height as _,
                rectangle.point.x as _,
                rectangle.point.y as _,
            );
        }
    }
}

impl Drop for XPainter {
    fn drop(&mut self) {
        unsafe {
            xlib::XFreeGC(self.display, self.gc);
            xlib::XFreePixmap(self.display, self.pixmap);
        }
    }
}

impl From<&xlib::XEvent> for XEvent {
    fn from(event: &xlib::XEvent) -> XEvent {
        use self::XEvent::*;

        match event.get_type() {
            xlib::MotionNotify => MotionNotify(xlib::XMotionEvent::from(event)),
            xlib::ButtonPress => ButtonPress(xlib::XButtonEvent::from(event)),
            xlib::ButtonRelease => ButtonRelease(xlib::XButtonEvent::from(event)),
            xlib::ColormapNotify => ColormapNotify(xlib::XColormapEvent::from(event)),
            xlib::EnterNotify => EnterNotify(xlib::XCrossingEvent::from(event)),
            xlib::LeaveNotify => LeaveNotify(xlib::XCrossingEvent::from(event)),
            xlib::Expose => Expose(xlib::XExposeEvent::from(event)),
            xlib::GraphicsExpose => GraphicsExpose(xlib::XGraphicsExposeEvent::from(event)),
            xlib::NoExpose => NoExpose(xlib::XNoExposeEvent::from(event)),
            xlib::FocusIn => FocusIn(xlib::XFocusChangeEvent::from(event)),
            xlib::FocusOut => FocusOut(xlib::XFocusChangeEvent::from(event)),
            xlib::KeymapNotify => KeymapNotify(xlib::XKeymapEvent::from(event)),
            xlib::KeyPress => KeyPress(xlib::XKeyEvent::from(event)),
            xlib::KeyRelease => KeyRelease(xlib::XKeyEvent::from(event)),
            xlib::PropertyNotify => PropertyNotify(xlib::XPropertyEvent::from(event)),
            xlib::ResizeRequest => ResizeRequest(xlib::XResizeRequestEvent::from(event)),
            xlib::CirculateNotify => CirculateNotify(xlib::XCirculateEvent::from(event)),
            xlib::ConfigureNotify => ConfigureNotify(xlib::XConfigureEvent::from(event)),
            xlib::DestroyNotify => DestroyNotify(xlib::XDestroyWindowEvent::from(event)),
            xlib::GravityNotify => GravityNotify(xlib::XGravityEvent::from(event)),
            xlib::MapNotify => MapNotify(xlib::XMapEvent::from(event)),
            xlib::ReparentNotify => ReparentNotify(xlib::XReparentEvent::from(event)),
            xlib::UnmapNotify => UnmapNotify(xlib::XUnmapEvent::from(event)),
            xlib::CreateNotify => CreateNotify(xlib::XCreateWindowEvent::from(event)),
            xlib::CirculateRequest => CirculateRequest(xlib::XCirculateRequestEvent::from(event)),
            xlib::ConfigureRequest => ConfigureRequest(xlib::XConfigureRequestEvent::from(event)),
            xlib::MapRequest => MapRequest(xlib::XMapRequestEvent::from(event)),
            xlib::ClientMessage => ClientMessage(xlib::XClientMessageEvent::from(event)),
            xlib::MappingNotify => MappingNotify(xlib::XMappingEvent::from(event)),
            xlib::SelectionClear => SelectionClear(xlib::XSelectionClearEvent::from(event)),
            xlib::SelectionNotify => SelectionNotify(xlib::XSelectionEvent::from(event)),
            xlib::SelectionRequest => SelectionRequest(xlib::XSelectionRequestEvent::from(event)),
            xlib::VisibilityNotify => VisibilityNotify(xlib::XVisibilityEvent::from(event)),
            _ => Any(xlib::XAnyEvent::from(event)),
        }
    }
}
