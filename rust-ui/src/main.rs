#[macro_use]
extern crate rust_ui;
extern crate x11;

use std::any::Any;
use std::env;
use std::ptr;

use x11::xlib;

use rust_ui::event::handler::EventContext;
use rust_ui::event::mouse::{MouseDown, MouseEvent};
use rust_ui::geometrics::WindowSize;
use rust_ui::platform::backend::Backend;
use rust_ui::platform::paint::GeneralPainter;
use rust_ui::platform::x11::backend::XBackend;
use rust_ui::platform::x11::error_handler;
use rust_ui::platform::x11::window;
use rust_ui::render::RenderContext;
use rust_ui::widget::element::Children;
use rust_ui::widget::fill::Fill;
use rust_ui::widget::flex::{Flex, FlexItem};
use rust_ui::widget::padding::Padding;
use rust_ui::widget::subscriber::Subscriber;
use rust_ui::widget::{Widget, WidgetMeta};

struct App;

impl App {
    fn on_click(&self, _event: &MouseEvent, state: &mut bool, context: &mut EventContext) {
        *state = !*state;

        context.notify_changes();
    }
}

impl<Painter: GeneralPainter + 'static> Widget<Painter> for App {
    type State = bool;

    fn render(
        &self,
        _children: Children<Painter>,
        state: &Self::State,
        context: &mut RenderContext<Self, Painter, Self::State>,
    ) -> Children<Painter> {
        element!(
            Subscriber::new().on(context.use_handler(MouseDown, Self::on_click)) => {
                Padding::uniform(32.0) => {
                    Flex::row() => {
                        if *state {
                            None
                        } else {
                            Some(element!(FlexItem::new(1.0).with_key(1) => Fill::new(0xff0000ff)))
                        },
                        FlexItem::new(1.0).with_key(2) => Fill::new(0x00ff00ff),
                        FlexItem::new(1.0).with_key(3) => Fill::new(0x0000ffff),
                    }
                }
            }
        )
        .into()
    }
}

impl WidgetMeta for App {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn main() {
    unsafe {
        error_handler::install();
    };

    let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
    if display.is_null() {
        panic!(
            "No display found at {}",
            env::var("DISPLAY").unwrap_or_default()
        );
    }

    let window_size = WindowSize {
        width: 640,
        height: 480,
    };
    let window = unsafe { window::create_window(display, window_size.width, window_size.height) };

    unsafe {
        xlib::XSelectInput(
            display,
            window,
            xlib::ButtonPressMask
                | xlib::ButtonReleaseMask
                | xlib::ExposureMask
                | xlib::StructureNotifyMask,
        );
        xlib::XMapWindow(display, window);
        xlib::XFlush(display);
    }

    let mut backend = XBackend::new(display, window);

    backend.run(window_size, element!(App));
}
