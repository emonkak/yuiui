use gtk::prelude::*;
use yuiui_core::{hlist, Atom, Effect, HigherOrderComponent, RenderContext, State, View};
use yuiui_gtk::views::{Button, Grid, GridChild, Label};
use yuiui_gtk::{EntryPoint, GtkElement};

#[derive(Debug, Default)]
struct AppState {
    count: Atom<i64>,
}

impl State for AppState {
    type Message = AppMessage;

    fn update(&mut self, message: Self::Message) -> Effect {
        match message {
            AppMessage::Increment => self.count.update(|count| *count += 1),
            AppMessage::Decrement => self.count.update(|count| *count -= 1),
        }
    }
}

#[derive(Debug)]
enum AppMessage {
    Increment,
    Decrement,
}

fn app(
    _props: &(),
    context: &mut RenderContext<AppState>,
) -> impl GtkElement<AppState, AppMessage> {
    let count = context.use_atom(|state| &state.count);
    Grid::new().hexpand(true).vexpand(true).el(hlist![
        GridChild::new(0, 0, 1, 1).el(Button::new()
            .hexpand(true)
            .vexpand(true)
            .on_click(|context| context.dispatch(AppMessage::Decrement))
            .el(Label::new()
                .label("-".to_owned())
                .halign(gtk::Align::Center)
                .el(()))),
        GridChild::new(1, 0, 1, 1,).el(Button::new()
            .hexpand(true)
            .vexpand(true)
            .on_click(|context| context.dispatch(AppMessage::Increment))
            .el(Label::new()
                .label("+".to_owned())
                .halign(gtk::Align::Center)
                .el(()))),
        GridChild::new(0, 1, 2, 1,).el(Label::new()
            .hexpand(true)
            .vexpand(true)
            .label(count.to_string())
            .el(())),
    ])
}

fn on_activate(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .default_width(320)
        .default_height(240)
        .build();
    let entry_point = EntryPoint::new(window);
    let element = app.el(());
    let state = AppState::default();
    entry_point.run(element, state);
}

fn main() {
    let application = gtk::Application::new(None, Default::default());
    application.connect_activate(on_activate);
    application.run();
}
