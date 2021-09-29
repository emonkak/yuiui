extern crate env_logger;

#[macro_use]
extern crate yuiui;

use std::env;
use std::rc::Rc;
use x11rb::xcb_ffi::XCBConnection;
use yuiui::application::{self, RenderLoop, Store};
use yuiui::geometrics::{PhysicalRectangle, Thickness};
use yuiui::graphics::{wgpu, xcb as xcb_graphics, Color};
use yuiui::text::fontconfig::FontLoader;
use yuiui::ui::{xcb, Window};
use yuiui::widget::{attribute, Children, Component, Element, ElementInstance};
use yuiui::widget_impl::button::Button;
use yuiui::widget_impl::flex::{Flex, FlexParam};
use yuiui::widget_impl::padding::Padding;

struct App;

impl<State: 'static, Message: 'static> Component<State, Message> for App {
    type LocalState = ();

    fn initial_state(&self) -> Self::LocalState {
        ()
    }

    fn render(
        &self,
        _children: &Children<State, Message>,
        _state: &Self::LocalState,
    ) -> Element<State, Message> {
        element!(
            Flex::column() => [
                Padding { thickness: Thickness::uniform(8.0) } => [
                    Button { background: Color::RED.into() },
                    attribute(FlexParam(1.0)),
                ]
                Padding { thickness: Thickness::uniform(8.0) } => [
                    Button { background: Color::GREEN.into() },
                    attribute(FlexParam(1.0)),
                ]
                Padding { thickness: Thickness::uniform(8.0) } => [
                    Button { background: Color::BLUE.into() },
                    attribute(FlexParam(1.0)),
                ]
            ]
        )
    }
}

impl<State: 'static, Message: 'static> From<App> for ElementInstance<State, Message> {
    fn from(component: App) -> ElementInstance<State, Message> {
        component.into_rc().into()
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
            x: 960,
            y: 240,
            width: 640,
            height: 480,
        },
        1.0,
    )
    .unwrap();

    window_container.window().show();

    let element: Element<usize, ()> = element!(App);
    let render_loop = RenderLoop::new(element);
    let store = Store::new(0, |counter, _| {
        *counter += 1;
        true
    });

    match env::var("RENDERER") {
        Ok(renderer_var) if renderer_var == "x11" => {
            let renderer =
                xcb_graphics::Renderer::new(connection, screen_num, window_container.window().id());
            application::run(render_loop, store, window_container, event_loop, renderer).unwrap();
        }
        _ => {
            let font_loader = FontLoader;
            let renderer = wgpu::Renderer::new(
                window_container.window().clone(),
                font_loader,
                wgpu::Settings::default(),
            )
            .unwrap();
            application::run(render_loop, store, window_container, event_loop, renderer).unwrap();
        }
    };
}
