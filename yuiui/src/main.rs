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
use yuiui::text::{FontDescriptor, Weight, HorizontalAlign, VerticalAlign};
use yuiui::ui::{xcb, Window};
use yuiui::widget::{
    attribute, Children, Command, Component, Effect, Element, ElementInstance, Event, EventMask,
    Lifecycle,
};
use yuiui::widget_impl::button::Button;
use yuiui::widget_impl::flex::{Flex, FlexParam};
use yuiui::widget_impl::label::Label;
use yuiui::widget_impl::padding::Padding;

struct App;

impl Component<State, Message> for App {
    type LocalState = State;

    fn initial_state(&self) -> Self::LocalState {
        State::default()
    }

    fn on_lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        _state: &mut Self::LocalState,
    ) -> Effect<Message> {
        match lifecycle {
            Lifecycle::Mounted => Effect::AddListener(EventMask::StateChanged.into()),
            _ => Effect::None,
        }
    }

    fn on_event(&self, event: Event<State>, state: &mut Self::LocalState) -> Effect<Message> {
        match event {
            Event::StateChanged(global_state) => {
                *state = *global_state;
                Command::RequestUpdate.into()
            }
            _ => Effect::None,
        }
    }

    fn render(
        &self,
        _children: &Children<State, Message>,
        state: &Self::LocalState,
    ) -> Element<State, Message> {
        element!(
            Flex::column() => [
                Padding { thickness: Thickness::uniform(8.0) } => [
                    attribute(FlexParam(1.0)),
                    Button {
                        background: Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }.into(),
                        on_click: Some(Box::new(|_| Command::Send(Message::Decrement).into()))
                    } => [
                        Label {
                            content: "-".to_owned(),
                            font: FontDescriptor {
                                weight: Weight::BOLD,
                                ..FontDescriptor::default()
                            },
                            font_size: 32.0,
                            horizontal_align: HorizontalAlign::Center,
                            vertical_align: VerticalAlign::Middle,
                            ..Label::default()
                        },
                    ]
                ]
                Padding { thickness: Thickness::uniform(8.0) } => [
                    attribute(FlexParam(1.0)),
                    Label {
                        content: format!("{}", state.count),
                        font_size: 32.0,
                        horizontal_align: HorizontalAlign::Center,
                        vertical_align: VerticalAlign::Middle,
                        ..Label::default()
                    },
                ]
                Padding { thickness: Thickness::uniform(8.0) } => [
                    attribute(FlexParam(1.0)),
                    Button {
                        background: Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }.into(),
                        on_click: Some(Box::new(|_| Command::Send(Message::Increment).into()))
                    } => [
                        Label {
                            content: "+".to_owned(),
                            font: FontDescriptor {
                                weight: Weight::BOLD,
                                ..FontDescriptor::default()
                            },
                            font_size: 32.0,
                            horizontal_align: HorizontalAlign::Center,
                            vertical_align: VerticalAlign::Middle,
                            ..Label::default()
                        },
                    ]
                ]
            ]
        )
    }
}

impl From<App> for ElementInstance<State, Message> {
    fn from(component: App) -> ElementInstance<State, Message> {
        component.into_rc().into()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct State {
    count: isize,
}

enum Message {
    Increment,
    Decrement,
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
            height: 240,
        },
        1.0,
    )
    .unwrap();

    window_container.window().show();

    let element = element!(App);
    let render_loop = RenderLoop::new(element);
    let store = Store::new(State { count: 0 }, |state, message| {
        match message {
            Message::Decrement => state.count -= 1,
            Message::Increment => state.count += 1,
        }
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
