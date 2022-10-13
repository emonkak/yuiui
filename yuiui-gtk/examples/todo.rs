use gtk::prelude::*;
use hlist::hlist;
use std::rc::Rc;
use yuiui::{Effect, HigherOrderComponent, Memoize, State, View};
use yuiui_gtk::views::{hbox, vbox, Button, Entry, Label, ListBox, ListBoxRow, ScrolledWindow};
use yuiui_gtk::{DefaultEntryPoint, EntryPoint, GtkElement};

#[derive(Debug, Default)]
struct AppState {
    todos: Vec<Rc<Todo>>,
    text: String,
}

impl State for AppState {
    type Message = AppMessage;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>) {
        match message {
            AppMessage::AddTodo(text) => {
                let todo = Todo {
                    id: self.todos.len(),
                    text,
                };
                self.todos.push(Rc::new(todo));
                self.text = "".to_owned();
                (true, Effect::new())
            }
            AppMessage::RemoveTodo(id) => {
                if let Some(position) = self.todos.iter().position(|todo| todo.id == id) {
                    self.todos.remove(position);
                }
                (true, Effect::new())
            }
            AppMessage::ChangeText(text) => {
                self.text = text;
                (true, Effect::new())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Todo {
    id: TodoId,
    text: String,
}

type TodoId = usize;

#[derive(Debug)]
enum AppMessage {
    AddTodo(String),
    RemoveTodo(TodoId),
    ChangeText(String),
}

fn todo_item(todo: &Todo) -> impl GtkElement<AppState, AppMessage> {
    let id = todo.id;
    hbox().hexpand(true).el_with(hlist![
        Label::new()
            .hexpand(true)
            .halign(gtk::Align::Start)
            .label(todo.text.to_owned())
            .el(),
        Button::new()
            .on_click(Box::new(move |_| AppMessage::RemoveTodo(id).into()))
            .el_with(Label::new().label("Delete".to_owned()).el())
    ])
}

fn todo_list(_props: &(), state: &AppState) -> impl GtkElement<AppState, AppMessage> {
    ListBox::new()
        .hexpand(true)
        .el_with(Vec::from_iter(state.todos.iter().map(|todo| {
            Memoize::new(
                |todo| ListBoxRow::new().hexpand(true).el_with(todo_item(todo)),
                todo.clone(),
            )
        })))
}

fn app(_props: &(), state: &AppState) -> impl GtkElement<AppState, AppMessage> {
    vbox().hexpand(true).vexpand(true).el_with(hlist![
        Entry::new()
            .text(state.text.to_owned())
            .hexpand(true)
            .on_activate(Box::new(
                |text, _| (!text.is_empty()).then(|| AppMessage::AddTodo(text.to_owned()))
            ))
            .on_change(Box::new(
                |text, _| AppMessage::ChangeText(text.to_owned()).into()
            ))
            .el(),
        ScrolledWindow::new()
            .hexpand(true)
            .vexpand(true)
            .el_with(todo_list.el()),
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
