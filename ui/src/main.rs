#[macro_use]
extern crate ui;
extern crate x11;

use std::env;
use std::mem;
use std::ptr;
use x11::xlib;

use ui::backend::x11::event::XEvent;
use ui::backend::x11::paint::XPainter;
use ui::backend::x11::window::{self, XWindowHandle};
use ui::geometrics::Size;
use ui::layout::BoxConstraints;
use ui::paint::PaintContext;
use ui::updater::Updater;
use ui::widget::fill::Fill;
use ui::widget::flex::{Flex, FlexItem};
use ui::widget::padding::Padding;
use ui::widget::widget::{WidgetMeta, Element};

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

    let window = unsafe { window::create_window(display, 640, 480) };
    let handle = XWindowHandle::new(display, window);

    let mut updater: Updater<XWindowHandle> = Updater::new();
    let mut paint_context = PaintContext::new(XPainter::new(&handle));

    updater.update(render());
    updater.render();
    updater.layout(BoxConstraints::tight(&Size {
        width: 640.0,
        height: 480.0
    }));

    updater.update(render2());
    updater.render();
    updater.layout(BoxConstraints::tight(&Size {
        width: 640.0,
        height: 480.0
    }));

    println!("{}", updater);

    handle.show();

    let mut event: xlib::XEvent = unsafe { mem::MaybeUninit::uninit().assume_init() };

    unsafe {
        xlib::XFlush(handle.display);
    }

    loop {
        unsafe {
            xlib::XNextEvent(handle.display, &mut event);
            match XEvent::from(&event) {
                XEvent::Expose(_) => {
                    updater.paint(&handle, &mut paint_context);
                },
                _ => (),
            }
        }
    }
}
