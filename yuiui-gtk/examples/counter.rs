use gtk::prelude::*;
use hlist::hlist;
use yuiui::{Effect, HigherOrderComponent, State, Store, View};
use yuiui_gtk::views::{Button, Grid, GridChild, Label};
use yuiui_gtk::{DefaultEntryPoint, EntryPoint, GtkElement};

#[derive(Debug, Default)]
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

fn app(_props: &(), store: &Store<AppState>) -> impl GtkElement<AppState, AppMessage> {
    Grid::new().hexpand(true).vexpand(true).el_with(hlist![
        GridChild::new(
            Button::new()
                .hexpand(true)
                .vexpand(true)
                .on_click(Box::new(|| AppMessage::Decrement)),
            0,
            0,
            1,
            1,
        )
        .el_with(
            Label::new()
                .label("-".to_owned())
                .halign(gtk::Align::Center)
                .el()
        ),
        GridChild::new(
            Button::new()
                .hexpand(true)
                .vexpand(true)
                .on_click(Box::new(|| AppMessage::Increment)),
            1,
            0,
            1,
            1,
        )
        .el_with(
            Label::new()
                .label("+".to_owned())
                .halign(gtk::Align::Center)
                .el()
        ),
        GridChild::new(
            Label::new()
                .hexpand(true)
                .vexpand(true)
                .label(store.count.to_string()),
            0,
            1,
            2,
            1,
        )
        .el(),
    ])
}

fn on_activate(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .default_width(320)
        .default_height(240)
        .build();
    let entry_point = DefaultEntryPoint::from(window);
    let element = app.el();
    let state = AppState::default();
    entry_point.boot(element, state);
}

fn main() {
    let application = gtk::Application::new(None, Default::default());
    application.connect_activate(on_activate);
    application.run();
}
