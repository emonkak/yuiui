extern crate env_logger;

#[macro_use]
extern crate yuiui_legacy;

use std::env;
use std::rc::Rc;
use x11rb::connection::Connection as _;
use x11rb::xcb_ffi::XCBConnection;
use yuiui_legacy::application::{self, RenderLoop, Store};
use yuiui_legacy::geometrics::{PhysicalRect, RectOutsets};
use yuiui_legacy::graphics::{wgpu, xcb as xcb_graphics, Color};
use yuiui_legacy::style::LayoutStyle;
use yuiui_legacy::text::fontconfig::FontLoader;
use yuiui_legacy::text::{FontDescriptor, FontWeight, HorizontalAlign, VerticalAlign};
use yuiui_legacy::ui::{xcb, Window};
use yuiui_legacy::widget::{
    Children, Command, Component, Effect, Element, ElementInstance, Event, EventMask, Lifecycle,
};
use yuiui_legacy::widget_impl::button::Button;
use yuiui_legacy::widget_impl::text::Text;
use yuiui_legacy::widget_impl::view::View;

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
            View::column(LayoutStyle::default()) => [
                View::row(LayoutStyle {
                    flex: 1.0,
                    padding: RectOutsets::uniform(8.0),
                    ..Default::default()
                }) => [
                    Button {
                        background: Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }.into(),
                        on_click: Some(Box::new(|_| Command::Send(Message::Decrement).into()))
                    } => [
                        Text {
                            content: "-".to_owned(),
                            font: FontDescriptor {
                                weight: FontWeight::BOLD,
                                ..FontDescriptor::default()
                            },
                            font_size: 32.0,
                            horizontal_align: HorizontalAlign::Center,
                            vertical_align: VerticalAlign::Middle,
                            ..Text::default()
                        },
                    ]
                ]
                View::row(LayoutStyle {
                    flex: 1.0,
                    padding: RectOutsets::uniform(8.0),
                    ..Default::default()
                }) => [
                    Text {
                        content: format!("{}", state.count),
                        font_size: 32.0,
                        horizontal_align: HorizontalAlign::Center,
                        vertical_align: VerticalAlign::Middle,
                        ..Text::default()
                    },
                ]
                View::row(LayoutStyle {
                    flex: 1.0,
                    padding: RectOutsets::uniform(8.0),
                    ..Default::default()
                }) => [
                    Button {
                        background: Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }.into(),
                        on_click: Some(Box::new(|_| Command::Send(Message::Increment).into()))
                    } => [
                        Text {
                            content: "+".to_owned(),
                            font: FontDescriptor {
                                weight: FontWeight::BOLD,
                                ..FontDescriptor::default()
                            },
                            font_size: 32.0,
                            horizontal_align: HorizontalAlign::Center,
                            vertical_align: VerticalAlign::Middle,
                            ..Text::default()
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
    let screen = &connection.setup().roots[screen_num];

    let event_loop = xcb::EventLoop::new(connection.clone(), screen_num);
    let window_container = xcb::Window::create_container(
        connection.clone(),
        screen_num,
        PhysicalRect {
            x: ((screen.width_in_pixels / 2) as u32).saturating_sub(640 / 2),
            y: ((screen.height_in_pixels / 2) as u32).saturating_sub(240 / 2),
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