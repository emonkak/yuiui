mod calculator;

use gtk::prelude::*;
use gtk::{gdk, pango};
use yuiui_core::{
    hlist, Atom, CancellableCommand, Effect, HigherOrderComponent, RenderContext, State, View,
};
use yuiui_gtk::views::{vbox, Button, Grid, GridChild, Label};
use yuiui_gtk::{EntryPoint, GtkElement};

use calculator::*;

#[derive(Debug)]
struct AppState {
    calculator: Atom<Calculator>,
}

impl AppState {
    fn new() -> Self {
        Self {
            calculator: Atom::new(Calculator::new()),
        }
    }
}

impl State for AppState {
    type Message = AppMessage;

    fn update(
        &mut self,
        message: Self::Message,
    ) -> (Effect, Vec<CancellableCommand<Self::Message>>) {
        match message {
            AppMessage::CalculatorAction(action) => {
                let effect = self
                    .calculator
                    .update(|calculator| calculator.update(action));
                (effect, Vec::new())
            }
        }
    }
}

#[derive(Debug)]
enum AppMessage {
    CalculatorAction(Action),
}

fn calculator_state(calculator: &Calculator) -> impl GtkElement<AppState, AppMessage> {
    vbox()
        .css_classes(vec!["calculator-state".to_owned()])
        .el(Label::new()
            .halign(gtk::Align::End)
            .ellipsize(pango::EllipsizeMode::Start)
            .label(calculator.display().to_string())
            .el(()))
}

fn num_button(digit: Digit) -> impl GtkElement<AppState, AppMessage> {
    Button::new()
        .css_classes(vec!["calculator-button".to_owned(), "is-number".to_owned()])
        .hexpand(true)
        .vexpand(true)
        .on_click(Box::new(move |_| {
            AppMessage::CalculatorAction(Action::Digit(digit)).into()
        }))
        .el(Label::new().label(digit.into_char().to_string()).el(()))
}

fn operator_button(operator: Operator) -> impl GtkElement<AppState, AppMessage> {
    Button::new()
        .css_classes(vec![
            "calculator-button".to_owned(),
            "is-".to_owned() + operator.name(),
        ])
        .hexpand(true)
        .vexpand(true)
        .on_click(Box::new(move |_| {
            AppMessage::CalculatorAction(Action::Operator(operator)).into()
        }))
        .el(Label::new().label(operator.into_char().to_string()).el(()))
}

fn clear_button() -> impl GtkElement<AppState, AppMessage> {
    Button::new()
        .css_classes(vec!["calculator-button".to_owned(), "is-clear".to_owned()])
        .on_click(Box::new(move |_| {
            AppMessage::CalculatorAction(Action::Clear).into()
        }))
        .hexpand(true)
        .vexpand(true)
        .el(Label::new().label("C".to_owned()).el(()))
}

fn negate_button() -> impl GtkElement<AppState, AppMessage> {
    Button::new()
        .css_classes(vec!["calculator-button".to_owned(), "is-negate".to_owned()])
        .hexpand(true)
        .vexpand(true)
        .on_click(Box::new(move |_| {
            AppMessage::CalculatorAction(Action::Negate).into()
        }))
        .el(Label::new().label("±".to_owned()).el(()))
}

fn dot_button() -> impl GtkElement<AppState, AppMessage> {
    Button::new()
        .css_classes(vec!["calculator-button".to_owned(), "is-dot".to_owned()])
        .hexpand(true)
        .vexpand(true)
        .on_click(Box::new(move |_| {
            AppMessage::CalculatorAction(Action::Dot).into()
        }))
        .el(Label::new().label(".".to_owned()).el(()))
}

fn equal_button() -> impl GtkElement<AppState, AppMessage> {
    Button::new()
        .css_classes(vec!["calculator-button".to_owned(), "is-equal".to_owned()])
        .hexpand(true)
        .vexpand(true)
        .on_click(Box::new(move |_| {
            AppMessage::CalculatorAction(Action::Equal).into()
        }))
        .el(Label::new().label("=".to_string()).el(()))
}

fn app(
    _props: &(),
    context: &mut RenderContext<AppState>,
) -> impl GtkElement<AppState, AppMessage> {
    let calculator = context.use_atom(|state| &state.calculator);
    Grid::new().hexpand(true).vexpand(true).el(hlist![
        GridChild::new(0, 0, 4, 1).el(calculator_state(calculator)),
        GridChild::new(0, 1, 1, 1).el(clear_button()),
        GridChild::new(1, 1, 1, 1).el(negate_button()),
        GridChild::new(2, 1, 1, 1).el(operator_button(Operator::Mod)),
        GridChild::new(3, 1, 1, 1).el(operator_button(Operator::Div)),
        GridChild::new(0, 2, 1, 1).el(num_button(Digit::Seven)),
        GridChild::new(1, 2, 1, 1).el(num_button(Digit::Eight)),
        GridChild::new(2, 2, 1, 1).el(num_button(Digit::Nine)),
        GridChild::new(3, 2, 1, 1).el(operator_button(Operator::Mul)),
        GridChild::new(0, 3, 1, 1).el(num_button(Digit::Four)),
        GridChild::new(1, 3, 1, 1).el(num_button(Digit::Five)),
        GridChild::new(2, 3, 1, 1).el(num_button(Digit::Six)),
        GridChild::new(3, 3, 1, 1).el(operator_button(Operator::Sub)),
        GridChild::new(0, 4, 1, 1).el(num_button(Digit::One)),
        GridChild::new(1, 4, 1, 1).el(num_button(Digit::Two)),
        GridChild::new(2, 4, 1, 1).el(num_button(Digit::Three)),
        GridChild::new(3, 4, 1, 1).el(operator_button(Operator::Add)),
        GridChild::new(0, 5, 2, 1).el(num_button(Digit::Zero)),
        GridChild::new(2, 5, 1, 1).el(dot_button()),
        GridChild::new(3, 5, 1, 1).el(equal_button()),
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
    let state = AppState::new();
    entry_point.run(element, state);
}

fn on_startup(_application: &gtk::Application) {
    let display = gdk::Display::default().expect("Could not connect to a display.");
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_bytes!("style.css"));

    gtk::StyleContext::add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() {
    let application = gtk::Application::new(None, Default::default());
    application.connect_startup(on_startup);
    application.connect_activate(on_activate);
    application.run();
}
