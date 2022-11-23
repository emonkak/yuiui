use gtk::prelude::*;
use hlist::hlist;
use std::rc::Rc;
use yuiui::{Atom, Effect, HigherOrderComponent, Memoize, RenderContext, State, View};
use yuiui_gtk::views::{hbox, vbox, Button, Entry, Label, ListBox, ListBoxRow, ScrolledWindow};
use yuiui_gtk::{EntryPoint, GtkElement};

#[derive(Debug, Default)]
struct AppState {
    todos: Atom<Vec<Rc<Todo>>>,
    text: Atom<String>,
    todo_id: usize,
}

impl State for AppState {
    type Message = AppMessage;

    fn update(&mut self, message: Self::Message) -> Effect<Self::Message> {
        match message {
            AppMessage::AddTodo(text) => {
                let todo = Todo {
                    id: self.todo_id,
                    text,
                };
                let subscribers = [
                    self.todos.update(|todos| {
                        todos.push(Rc::new(todo));
                    }),
                    self.text.update(|text| {
                        *text = "".to_owned();
                    }),
                ]
                .into_iter()
                .flatten()
                .collect();
                self.todo_id += 1;
                Effect::Update(subscribers)
            }
            AppMessage::RemoveTodo(id) => {
                if let Some(position) = self.todos.peek().iter().position(|todo| todo.id == id) {
                    let subscribers = self.todos.update(move |todos| {
                        todos.remove(position);
                    });
                    Effect::Update(subscribers)
                } else {
                    Effect::nop()
                }
            }
            AppMessage::ChangeText(new_text) => {
                let subscribers = self.text.update(move |text| {
                    *text = new_text;
                });
                Effect::Update(subscribers)
            }
        }
    }
}

#[derive(Debug)]
struct Todo {
    id: TodoId,
    text: String,
}

#[derive(Debug)]
struct TodoProps {
    todo: Rc<Todo>,
}

impl PartialEq for TodoProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.todo, &other.todo)
    }
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
    hbox().hexpand(true).el(hlist![
        Label::new()
            .hexpand(true)
            .halign(gtk::Align::Start)
            .label(todo.text.to_owned())
            .el(()),
        Button::new()
            .on_click(Box::new(move |_| AppMessage::RemoveTodo(id).into()))
            .el(Label::new().label("Delete".to_owned()).el(()))
    ])
}

fn todo_list(
    _props: &(),
    context: &mut RenderContext<AppState>,
) -> impl GtkElement<AppState, AppMessage> {
    let todos = context.use_atom(|state| &state.todos);
    ListBox::new().hexpand(true).el(todos
        .iter()
        .map(|todo| {
            Memoize::new(
                |props: &TodoProps, _context: &mut RenderContext<AppState>| {
                    ListBoxRow::new().hexpand(true).el(todo_item(&props.todo))
                },
                TodoProps { todo: todo.clone() },
            )
        })
        .collect::<Vec<_>>())
}

fn app(
    _props: &(),
    context: &mut RenderContext<AppState>,
) -> impl GtkElement<AppState, AppMessage> {
    let text = context.use_atom(|state| &state.text);
    vbox().hexpand(true).vexpand(true).el(hlist![
        Entry::new()
            .text(text.to_owned())
            .hexpand(true)
            .on_activate(Box::new(
                |text, _| (!text.is_empty()).then(|| AppMessage::AddTodo(text.to_owned()))
            ))
            .on_change(Box::new(
                |text, _| AppMessage::ChangeText(text.to_owned()).into()
            ))
            .el(()),
        ScrolledWindow::new()
            .hexpand(true)
            .vexpand(true)
            .el(todo_list.el(())),
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
