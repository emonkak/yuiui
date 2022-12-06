use gtk::prelude::*;
use std::rc::Rc;
use yuiui_core::{
    hlist, Atom, Effect, HigherOrderComponent, RenderContext, State, View, ViewElement,
};
use yuiui_gtk::views::{hbox, vbox, Button, Entry, Label, ListBox, ListBoxRow, ScrolledWindow};
use yuiui_gtk::{EntryPoint, GtkElement};

#[derive(Debug, Default)]
struct TodoState {
    todos: Atom<Vec<Rc<Todo>>>,
    text: Atom<String>,
    todo_id: usize,
}

impl State for TodoState {
    type Message = TodoMessage;

    fn update(&mut self, message: Self::Message) -> Effect {
        match message {
            TodoMessage::AddTodo(text) => {
                let todo = Todo {
                    id: self.todo_id,
                    text,
                };
                self.todos
                    .update(|todos| {
                        todos.push(Rc::new(todo));
                    })
                    .compose(self.text.set("".to_owned()))
            }
            TodoMessage::RemoveTodo(id) => {
                if let Some(position) = self.todos.get().iter().position(|todo| todo.id == id) {
                    self.todos.update(move |todos| {
                        todos.remove(position);
                    })
                } else {
                    Effect::Nop
                }
            }
            TodoMessage::ChangeText(new_text) => self.text.update(move |text| {
                *text = new_text;
            }),
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
enum TodoMessage {
    AddTodo(String),
    RemoveTodo(TodoId),
    ChangeText(String),
}

fn app(
    _props: &(),
    _context: &mut RenderContext<TodoState>,
) -> impl GtkElement<TodoState, TodoMessage> {
    vbox().hexpand(true).vexpand(true).el(hlist![
        todo_input.el(()),
        ScrolledWindow::new()
            .hexpand(true)
            .vexpand(true)
            .el(todo_list.el(())),
    ])
}

fn todo_input(
    _props: &(),
    context: &mut RenderContext<TodoState>,
) -> impl GtkElement<TodoState, TodoMessage> {
    let text = context.use_atom(|state| &state.text);
    Entry::new()
        .text(text.to_owned())
        .hexpand(true)
        .on_activate(|text, context| {
            if !text.is_empty() {
                context.dispatch(TodoMessage::AddTodo(text.to_owned()))
            }
        })
        .on_change(|text, context| context.dispatch(TodoMessage::ChangeText(text.to_owned())))
        .el(())
}

fn todo_list(
    _props: &(),
    context: &mut RenderContext<TodoState>,
) -> impl GtkElement<TodoState, TodoMessage> {
    let todos = context.use_atom(|state| &state.todos);
    ListBox::new().hexpand(true).el(todos
        .iter()
        .map(|todo| todo_item.memoize(TodoProps { todo: todo.clone() }))
        .collect::<Vec<_>>())
}

fn todo_item(
    props: &TodoProps,
    _context: &mut RenderContext<TodoState>,
) -> ViewElement<
    ListBoxRow<impl GtkElement<TodoState, TodoMessage>>,
    TodoState,
    TodoMessage,
    EntryPoint,
> {
    let id = props.todo.id;
    ListBoxRow::new()
        .hexpand(true)
        .el(hbox().hexpand(true).el(hlist![
            Label::new()
                .hexpand(true)
                .halign(gtk::Align::Start)
                .label(props.todo.text.to_owned())
                .el(()),
            Button::new()
                .on_click(move |context| context.dispatch(TodoMessage::RemoveTodo(id)))
                .el(Label::new().label("Delete".to_owned()).el(()))
        ]))
}

fn on_activate(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .default_width(320)
        .default_height(240)
        .build();
    let entry_point = EntryPoint::new(window);
    let element = app.el(());
    let state = TodoState::default();
    entry_point.run(element, state);
}

fn main() {
    let application = gtk::Application::new(None, Default::default());
    application.connect_activate(on_activate);
    application.run();
}
