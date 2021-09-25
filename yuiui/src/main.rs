extern crate env_logger;

#[macro_use]
extern crate yuiui;

use std::env;
use std::rc::Rc;
use x11rb::xcb_ffi::XCBConnection;
use yuiui::application;
use yuiui::geometrics::PhysicalRectangle;
use yuiui::graphics::{wgpu, xcb as xcb_graphics, Color};
use yuiui::text::fontconfig::FontLoader;
use yuiui::ui::{xcb, Window};
use yuiui::widget::{attribute, Component, ComponentExt, Element, ElementNode};
use yuiui::widget_impl::button::Button;
use yuiui::widget_impl::flex::{Flex, FlexParam};

struct App;

impl Component for App {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Self::State::default()
    }

    fn render(&self, _children: &Vec<Element>, _state: &Self::State) -> Element {
        element!(
            Flex::row() => [
                Button { background: Color::RED.into() } => attribute(FlexParam(1.0)),
                Button { background: Color::GREEN.into() } => attribute(FlexParam(1.0)),
                Button { background: Color::BLUE.into() } => attribute(FlexParam(1.0)),
            ]
        )
    }
}

impl From<App> for ElementNode {
    fn from(component: App) -> ElementNode {
        component.into_boxed().into()
    }
}

fn main() {
    env_logger::init();

    let (connection, screen_num) = XCBConnection::connect(None).unwrap();
    let connection = Rc::new(connection);

    let event_loop = xcb::EventLoop::new(connection.clone());
    let window_container = xcb::Window::create_container(
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

    let el = element!(App);

    match env::var("RENDERER") {
        Ok(renderer_var) if renderer_var == "x11" => {
            let renderer =
                xcb_graphics::Renderer::new(connection, screen_num, window_container.window().id());
            application::run(window_container, event_loop, renderer, el).unwrap();
        }
        _ => {
            let font_loader = FontLoader;
            let renderer = wgpu::Renderer::new(
                window_container.window().clone(),
                font_loader,
                wgpu::Settings::default(),
            )
            .unwrap();
            application::run(window_container, event_loop, renderer, el).unwrap();
        }
    };
}