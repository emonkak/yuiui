extern crate env_logger;
extern crate rust_ui;
extern crate x11;

use rust_ui::geometrics::{PhysicalRectangle, Rectangle};
use rust_ui::graphics::{wgpu, xcb as xcb_graphics, Color, Primitive};
use rust_ui::text::fontconfig::FontLoader;
use rust_ui::text::{FontDescriptor, FontFamily, FontWeight, HorizontalAlign, VerticalAlign};
use rust_ui::ui::application;
use rust_ui::ui::xcb;
use rust_ui::ui::Window;
use rust_ui::widget::element::{Children, ElementId, IntoElement};
use rust_ui::widget::fill::Fill;
use rust_ui::widget::flex::Flex;
use rust_ui::widget::padding::Padding;
use rust_ui::widget::text::Text;
use rust_ui::widget::{MessageSink, StateContainer, Widget, WidgetSeal};
use std::any::Any;
use std::env;
use std::rc::Rc;
use x11rb::xcb_ffi::XCBConnection;

#[derive(Debug)]
struct App {
    message: String,
}

impl<Renderer: 'static> Widget<Renderer> for App {
    type State = usize;
    type Message = ();

    fn initial_state(&self) -> StateContainer<Renderer, Self, Self::State, Self::Message> {
        StateContainer::from_pure_state(0)
    }

    fn update(
        &self,
        state: &mut Self::State,
        _event: &Self::Message,
        _messages: &mut MessageSink,
    ) -> bool {
        *state += 1;
        true
    }

    fn render(&self, _state: &Self::State, _element_id: ElementId) -> Children<Renderer> {
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
            .add(
                Fill::new(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                1.0,
            )
            .add(
                Fill::new(Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                }),
                1.0,
            )
            .add(
                Text {
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
                },
                1.0,
            );

        Padding::uniform(16.0, column).into_element().into()
    }

    fn draw(
        &self,
        _state: &mut Self::State,
        _bounds: Rectangle,
        _renderer: &mut Renderer,
        _messages: &mut MessageSink,
    ) -> Option<Primitive> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WidgetSeal for App {}

fn main() {
    env_logger::init();

    let (connection, screen_num) = XCBConnection::connect(None).unwrap();
    let connection = Rc::new(connection);

    let event_loop = xcb::EventLoop::new(connection.clone());
    let window_container = xcb::Window::create(
        connection.clone(),
        screen_num,
        PhysicalRectangle {
            x: 0,
            y: 0,
            width: 640,
            height: 480,
        },
        1.0,
    )
    .unwrap();

    window_container.window().show();

    let app = App {
        message:
            "QSとはQuality Startの略であり、1985年にスポーツライターJohn Loweにより提唱された。"
                .to_string(),
    };

    match env::var("RENDERER") {
        Ok(renderer_var) if renderer_var == "x11" => {
            let renderer =
                xcb_graphics::Renderer::new(connection, screen_num, window_container.window().id());
            application::run(event_loop, renderer, window_container, app.into_element()).unwrap();
        }
        _ => {
            let font_loader = FontLoader;
            let renderer = wgpu::Renderer::new(
                window_container.window().clone(),
                font_loader,
                wgpu::Settings::default(),
            )
            .unwrap();
            application::run(event_loop, renderer, window_container, app.into_element()).unwrap();
        }
    };
}
