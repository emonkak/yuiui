#[macro_use]
extern crate rust_ui;
extern crate x11;
extern crate env_logger;

use std::any::Any;
use std::env;
use std::ptr;
use x11::xlib;

use rust_ui::event::handler::EventContext;
use rust_ui::event::mouse::{MouseDown, MouseEvent};
use rust_ui::geometrics::{PhysicalPoint, Size};
use rust_ui::graphics::{Color, Viewport, wgpu, x11 as x11_graphics};
use rust_ui::render::RenderContext;
use rust_ui::text::fontconfig::FontLoader;
use rust_ui::text::{FontDescriptor, FontFamily, FontWeight, HorizontalAlign, VerticalAlign};
use rust_ui::ui::Window;
use rust_ui::ui::application;
use rust_ui::ui::x11 as x11_ui;
use rust_ui::widget::element::Children;
use rust_ui::widget::fill::Fill;
use rust_ui::widget::flex::{Flex, FlexItem};
use rust_ui::widget::padding::Padding;
use rust_ui::widget::subscriber::Subscriber;
use rust_ui::widget::text::Text;
use rust_ui::widget::{Widget, WidgetMeta};

struct App;

impl App {
    fn on_click(&self, _event: &MouseEvent, state: &mut bool, context: &mut EventContext) {
        *state = !*state;

        context.notify_changes();
    }
}

impl<Renderer: 'static> Widget<Renderer> for App {
    type State = bool;

    fn render(
        &self,
        _children: Children<Renderer>,
        state: &Self::State,
        context: &mut RenderContext<Self, Renderer, Self::State>,
    ) -> Children<Renderer> {
        element!(
            Subscriber::new().on(context.use_handler::<MouseDown>(Self::on_click)) => {
                Padding::uniform(32.0) => {
                    Flex::column() => {
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
                            Padding::uniform(16.0) => Text {
                                content: "QSとはQuality Startの略であり、1985年にスポーツライター John Lowe により提唱された。".to_owned(),
                                color: Color::BLACK,
                                font: FontDescriptor {
                                    family: FontFamily::SansSerif,
                                    weight: FontWeight::BOLD,
                                    ..FontDescriptor::default()
                                },
                                font_size: 16.0,
                                horizontal_align: HorizontalAlign::Center,
                                vertical_align: VerticalAlign::Middle,
                            }
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
    env_logger::init();

    unsafe {
        x11_ui::install_error_handler();
    };

    let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
    if display.is_null() {
        panic!(
            "No display found at {}",
            env::var("DISPLAY").unwrap_or_default()
        );
    }

    let viewport = Viewport::from_logical(Size {
        width: 640.0,
        height: 480.0,
    }, 2.0);
    let event_loop = x11_ui::EventLoop::create(display).unwrap();
    let window = x11_ui::Window::create(display, viewport, PhysicalPoint {
        x: 0,
        y: 0,
    });

    window.show();

    match env::var("RENDERER") {
        Ok(renderer_var) if renderer_var == "x11" => {
            let renderer = x11_graphics::Renderer::new(display, window.window_id());
            application::run(event_loop, renderer, window, element!(App));
        }
        _ => {
            let font_loader = FontLoader;
            let renderer =
                wgpu::Renderer::new(window.clone(), font_loader, wgpu::Settings::default())
                    .unwrap();
            application::run(event_loop, renderer, window, element!(App));
        }
    };
}
