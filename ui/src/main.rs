extern crate ui;
extern crate x11;

use std::mem;
use x11::xlib;

use ui::ui::{UIMain, UIState};
use ui::widget::fill::Fill;
use ui::widget::flex::Column;
use ui::widget::padding::Padding;
use ui::window::WindowHandle;
use ui::window::x11::{XWindowHandle, XWindowProcedure, XPaintContext};

fn build_ui(ui: &mut UIState<XWindowHandle, XPaintContext>) {
    let mut row = Column::new();
    let children = &[
        Fill::new(0xff0000ff).ui(ui),
        Fill::new(0x00ff00ff).ui(ui),
        Fill::new(0x0000ffff).ui(ui),
    ];
    for child in children {
        row.set_flex(*child, 1.0);
    }

    let root = Padding::uniform(5.0).ui(row.ui(children, ui), ui);
    ui.set_root(root);
}

fn main() {
    let handle = XWindowHandle::new(640, 480).unwrap();
    let mut state = UIState::new(handle.clone());

    build_ui(&mut state);

    let window_proc = XWindowProcedure {
        handler: Box::new(UIMain::new(state)),
        handle: handle.clone()
    };

    handle.show();

    let mut event: xlib::XEvent = unsafe { mem::MaybeUninit::uninit().assume_init() };

    unsafe {
        xlib::XFlush(handle.display);
    }

    loop {
        unsafe {
            xlib::XNextEvent(handle.display, &mut event);
            if !window_proc.dispatch_event(&event) {
                break;
            }
        }
    }
}
