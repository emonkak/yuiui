#[macro_use]
extern crate rust_ui;
extern crate x11;

use std::env;
use std::mem;
use std::ptr;
use x11::xlib;

use rust_ui::geometrics::{Point, Rectangle, Size};
use rust_ui::paint::PaintContext;
use rust_ui::platform::WindowHandle;
use rust_ui::platform::x11::event::XEvent;
use rust_ui::platform::x11::paint::XPaintContext;
use rust_ui::platform::x11::window::{self, XWindowHandle};
use rust_ui::updater::Updater;
use rust_ui::widget::fill::Fill;
use rust_ui::widget::flex::{Flex, FlexItem};
use rust_ui::widget::padding::Padding;
use rust_ui::widget::{WidgetMeta, Element};

fn render() -> Element<XWindowHandle> {
    element!(
        Padding::uniform(32.0) => {
            Flex::row() => {
                FlexItem::new(1.0).with_key(1) => { Fill::new(0xff0000ff) }
                FlexItem::new(1.0).with_key(2) => { Fill::new(0x0000ffff) }
                FlexItem::new(1.0).with_key(3) => { Fill::new(0x00ff00ff) }
                FlexItem::new(1.0).with_key(4) => { Fill::new(0x00ffffff) }
            }
        }
    )
}

fn render2() -> Element<XWindowHandle> {
    element!(
        Padding::uniform(32.0) => {
            Flex::row() => {
                FlexItem::new(1.0).with_key(2) => { Fill::new(0x0000ffff) }
                FlexItem::new(1.0).with_key(1) => { Fill::new(0xff0000ff) }
                FlexItem::new(1.0).with_key(4) => { Fill::new(0x00ffffff) }
                FlexItem::new(1.0).with_key(3) => { Fill::new(0x00ff00ff) }
            }
        }
    )
}

fn main() {
    let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
    if display.is_null() {
        panic!(
            "No display found at {}",
            env::var("DISPLAY").unwrap_or_default()
        );
    }

    let mut window_width: u32 = 640;
    let mut window_height: u32 = 480;
    let window = unsafe { window::create_window(display, window_width, window_height) };
    let handle = XWindowHandle::new(display, window);

    let mut updater: Updater<XWindowHandle> = Updater::new();

    updater.update(render());
    updater.render();
    updater.layout(Size { width: window_width as _, height: window_height as _ }, false);

    updater.update(render2());
    updater.render();
    updater.layout(Size { width: window_width as _, height: window_height as _ }, false);

    let mut event: xlib::XEvent = unsafe { mem::MaybeUninit::uninit().assume_init() };
    let mut paint_context = XPaintContext::new(&handle);

    updater.paint(&handle, &mut paint_context);

    handle.show_window();

    unsafe {
        xlib::XFlush(handle.display);
    }

    loop {
        unsafe {
            xlib::XNextEvent(handle.display, &mut event);
            match XEvent::from(&event) {
                XEvent::Expose(_) => {
                    paint_context.commit(&handle, &Rectangle {
                        point: Point::ZERO,
                        size: Size {
                            width: window_width as _,
                            height: window_height as _,
                        }
                    });
                },
                XEvent::ConfigureNotify(event) => {
                    if window_width != event.width as _ || window_height != event.height as _ {
                        window_width = event.width as _;
                        window_height = event.height as _;

                        paint_context = XPaintContext::new(&handle);
                        updater.layout(Size { width: window_width as _, height: window_height as _ }, true);

                        updater.paint(&handle, &mut paint_context);

                        paint_context.commit(&handle, &Rectangle {
                            point: Point::ZERO,
                            size: Size {
                                width: window_width as _,
                                height: window_height as _,
                            }
                        });
                    }
                },
                _ => (),
            }
        }
    }
}
