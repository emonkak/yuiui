use gtk::prelude::*;
use hlist::hlist;
use yuiui::{Effect, HigherOrderComponent, State, Store, View};
use yuiui_gtk::views::{hbox, vbox, Button, Entry, Label, ListBox, ListBoxRow, ScrolledWindow};
use yuiui_gtk::{DefaultEntryPoint, EntryPoint, GtkElement, Renderer};

#[derive(Debug, Default)]
struct AppState {
    todos: Vec<Todo>,
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
                self.todos.push(todo);
                self.text = "".to_owned();
                (true, Effect::none())
            }
            AppMessage::RemoveTodo(id) => {
                if let Some(position) = self.todos.iter().position(|todo| todo.id == id) {
                    self.todos.remove(position);
                }
                (true, Effect::none())
            }
            AppMessage::ChangeText(text) => {
                self.text = text;
                (true, Effect::none())
            }
        }
    }
}

#[derive(Debug)]
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

fn todo_list(
    _props: &(),
    store: &Store<AppState>,
) -> impl GtkElement<AppState, AppMessage, Renderer> {
    ListBox::new()
        .hexpand(true)
        .el_with(Vec::from_iter(store.todos.iter().map(|todo| {
            let id = todo.id;
            ListBoxRow::new()
                .hexpand(true)
                .el_with(hbox().hexpand(true).el_with(hlist![
                            Label::new()
                                .hexpand(true)
                                .halign(gtk::Align::Start)
                                .label(todo.text.to_owned())
                                .el(),
                            Button::new()
                                .on_click(Box::new(move |_| AppMessage::RemoveTodo(id)))
                                .el_with(Label::new().label("Delete".to_owned()).el())
                        ]))
        })))
}

fn app(_props: &(), store: &Store<AppState>) -> impl GtkElement<AppState, AppMessage, Renderer> {
    vbox().hexpand(true).vexpand(true).el_with(hlist![
        Entry::new()
            .text(store.text.to_owned())
            .hexpand(true)
            .on_activate(Box::new(|text, _| AppMessage::AddTodo(text.to_owned())))
            .on_change(Box::new(|text, _| AppMessage::ChangeText(text.to_owned())))
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
