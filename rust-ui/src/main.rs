#[macro_use]
extern crate rust_ui;
extern crate x11;

use std::any::Any;
use std::env;
use std::ptr;

use x11::xlib;

use rust_ui::event::handler::EventContext;
use rust_ui::event::mouse::{MouseDown, MouseEvent};
use rust_ui::graphics::{wgpu as graphics, Color};
use rust_ui::render::context::RenderContext;
use rust_ui::ui::application;
use rust_ui::ui::x11::error_handler;
use rust_ui::ui::x11::event_loop::XEventLoop;
use rust_ui::ui::x11::window::XWindow;
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

impl Widget<graphics::Renderer> for App {
    type State = bool;

    fn render(
        &self,
        _children: Children<graphics::Renderer>,
        state: &Self::State,
        context: &mut RenderContext<Self, graphics::Renderer, Self::State>,
    ) -> Children<graphics::Renderer> {
        element!(
            Subscriber::new().on(context.use_handler::<MouseDown>(Self::on_click)) => {
                Padding::uniform(32.0) => {
                    Flex::row() => {
                        if *state {
                            None
                        } else {
                            Some(element!(FlexItem::new(1.0).with_key(1) => {
                                Padding::uniform(16.0) => Fill::new(Color {
                                    r: 1.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                })
                            }))
                        },
                        FlexItem::new(1.0).with_key(2) => {
                            Padding::uniform(16.0) => Fill::new(Color {
                                r: 0.0,
                                g: 1.0,
                                b: 0.0,
                                a: 1.0,
                            })
                        }
                        FlexItem::new(1.0).with_key(3) => {
                            Padding::uniform(16.0) => Fill::new(Color {
                                r: 0.0,
                                g: 0.0,
                                b: 1.0,
                                a: 1.0,
                            })
                        }
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

    let event_loop = XEventLoop::new(display).unwrap();
    let window = XWindow::new(display, 640, 480);

    unsafe {
        xlib::XSelectInput(
            display,
            window.window,
            xlib::ButtonPressMask
                | xlib::ButtonReleaseMask
                | xlib::ExposureMask
                | xlib::StructureNotifyMask,
        );
        xlib::XMapWindow(display, window.window);
        xlib::XFlush(display);
    }

    let renderer = graphics::Renderer::new(&window, graphics::Settings::default()).unwrap();

    application::run(event_loop, renderer, window, element!(App));
}
