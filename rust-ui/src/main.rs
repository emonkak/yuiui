extern crate rust_ui;
extern crate env_logger;
extern crate x11;

use std::any::Any;
use std::env;
use std::ptr;
use x11::xlib;

use rust_ui::geometrics::{PhysicalPoint, Rectangle, Size};
use rust_ui::graphics::{wgpu, x11 as x11_graphics, Color, Primitive, Viewport};
use rust_ui::text::fontconfig::FontLoader;
use rust_ui::text::{FontDescriptor, FontFamily, FontWeight, HorizontalAlign, VerticalAlign};
use rust_ui::ui::Window;
use rust_ui::ui::application;
use rust_ui::ui::x11 as x11_ui;
use rust_ui::widget::element::{Children, ElementId, IntoElement};
use rust_ui::widget::fill::Fill;
use rust_ui::widget::flex::{Flex};
use rust_ui::widget::message::{MessageEmitter, MessageQueue};
use rust_ui::widget::padding::Padding;
use rust_ui::widget::text::Text;
use rust_ui::widget::{AsAny, Widget};

#[derive(Debug)]
struct App {
    message: String,
}

impl<Renderer: 'static> Widget<Renderer> for App {
    type State = usize;
    type Message = ();

    fn update(
        &self,
        state: &mut Self::State,
        _event: &Self::Message,
        _message_queue: &mut MessageQueue,
    ) -> bool {
        *state += 1;
        true
    }

    fn render(
        &self,
        _state: &Self::State,
        _element_id: ElementId,
    ) -> Children<Renderer> {
        // element!(
        //     Padding::uniform(32.0) => {
        //         Flex::column() => {
        //             if *state % 2 == 0 {
        //                 None
        //             } else {
        //                 Some(element!(FlexItem::new(1.0).with_key(1) => {
        //                     Padding::uniform(16.0) => Fill::new(Color {
        //                         r: 1.0,
        //                         g: 0.0,
        //                         b: 0.0,
        //                         a: 1.0,
        //                     })
        //                 }))
        //             },
        //             FlexItem::new(1.0).with_key(2) => {
        //                 Padding::uniform(16.0) => Fill::new(Color {
        //                     r: 0.0,
        //                     g: 1.0,
        //                     b: 0.0,
        //                     a: 1.0,
        //                 })
        //             }
        //             FlexItem::new(1.0).with_key(3) => {
        //                 Padding::uniform(16.0) => Text {
        //                     content: self.message.clone(),
        //                     color: Color::BLACK,
        //                     font: FontDescriptor {
        //                         family: FontFamily::SansSerif,
        //                         weight: FontWeight::BOLD,
        //                         ..FontDescriptor::default()
        //                     },
        //                     font_size: 16.0,
        //                     horizontal_align: HorizontalAlign::Center,
        //                     vertical_align: VerticalAlign::Middle,
        //                 }
        //             }
        //         }
        //     }

        let column = Flex::column()
            .add(Fill::new(Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }), 1.0)
            .add(Fill::new(Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }), 1.0)
            .add(Text {
                content: self.message.clone(),
                color: Color::BLACK,
                font: FontDescriptor {
                    family: FontFamily::SansSerif,
                    weight: FontWeight::BOLD,
                    ..FontDescriptor::default()
                },
                font_size: 16.0,
                horizontal_align: HorizontalAlign::Center,
                vertical_align: VerticalAlign::Middle,
            }, 1.0);

        vec![Padding::uniform(16.0, column).into_element()]
    }

    fn draw(
        &self,
        _state: &mut Self::State,
        _bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter<Self::Message>
    ) -> Option<Primitive> {
        None
    }
}

impl AsAny for App {
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

    let viewport = Viewport::from_logical(
        Size {
            width: 640.0,
            height: 480.0,
        },
        1.0,
    );
    let event_loop = x11_ui::EventLoop::create(display).unwrap();
    let window = x11_ui::Window::create(display, viewport, PhysicalPoint { x: 0, y: 0 });

    window.show();

    let app = App {
        message:
            "QSとはQuality Startの略であり、1985年にスポーツライターJohn Loweにより提唱された。"
                .to_string(),
    };

    match env::var("RENDERER") {
        Ok(renderer_var) if renderer_var == "x11" => {
            let renderer = x11_graphics::Renderer::new(display, window.window_id());
            application::run(event_loop, renderer, window, app.into_element());
        }
        _ => {
            let font_loader = FontLoader;
            let renderer =
                wgpu::Renderer::new(window.clone(), font_loader, wgpu::Settings::default())
                    .unwrap();
            application::run(event_loop, renderer, window, app.into_element());
        }
    };
}
