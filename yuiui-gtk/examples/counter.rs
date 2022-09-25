use gtk::prelude::*;
use hlist::hlist;
use yuiui::{Component, ComponentEl, Effect, FunctionComponent, State, Store};
use yuiui_gtk::widgets::{button, label, r#box, BoxBuilder, ButtonBuilder, LabelBuilder};
use yuiui_gtk::{Application, GtkBackend, GtkElement};

fn main() {
    let application = gtk::Application::new(None, Default::default());
    application.connect_activate(on_activate);
    application.run();
}

fn on_activate(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .default_width(320)
        .default_height(240)
        .build();

    let application = Application::new(application.clone(), window.clone());
    let element = app();
    let store = Store::new(AppState::default());

    application.start(element, store);

    window.show();
}

fn app() -> ComponentEl<
    FunctionComponent<(), impl GtkElement<AppState, AppMessage>, AppState, AppMessage, GtkBackend>,
> {
    FunctionComponent::new((), |_props, state: &AppState| {
        r#box(
            BoxBuilder::new().orientation(gtk::Orientation::Vertical),
            hlist![
                r#box(
                    BoxBuilder::new().orientation(gtk::Orientation::Horizontal),
                    hlist![
                        button(
                            ButtonBuilder::new(),
                            |_| AppMessage::Decrement,
                            label(
                                LabelBuilder::new()
                                    .label("-".to_owned())
                                    .halign(gtk::Align::Center)
                            )
                        ),
                        button(
                            ButtonBuilder::new(),
                            |_| AppMessage::Increment,
                            label(
                                LabelBuilder::new()
                                    .label("+".to_owned())
                                    .halign(gtk::Align::Center)
                            )
                        ),
                    ]
                ),
                label(LabelBuilder::new().label(state.count.to_string()))
            ],
        )
    })
    .el()
}

#[derive(Default)]
struct AppState {
    count: i64,
}

impl State for AppState {
    type Message = AppMessage;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>) {
        match message {
            AppMessage::Increment => {
                self.count += 1;
            }
            AppMessage::Decrement => {
                self.count -= 1;
            }
        }
        (true, Effect::none())
    }
}

#[derive(Debug)]
enum AppMessage {
    Increment,
    Decrement,
}
