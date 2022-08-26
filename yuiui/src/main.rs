use either_macro::either;
use hlist::hlist;
use std::fmt::Debug;

use yuiui::*;

#[derive(Debug)]
struct AppState {
    count: Data<i64>,
}

#[derive(Debug)]
struct AppEnv {}

#[allow(dead_code)]
#[derive(Debug)]
enum AppMessage {
    Increment,
    Decrement,
}

impl State for AppState {
    type Message = AppMessage;

    fn reduce(&mut self, message: AppMessage) -> bool {
        match message {
            AppMessage::Increment => self.count.value += 1,
            AppMessage::Decrement => self.count.value -= 1,
        }
        true
    }
}

fn app(_state: &AppState) -> Element![AppState, AppEnv] {
    Block::new().el_with(hlist![
        Block::new().el_with(vec![Text::new("hello").el(), Text::new("world").el()]),
        Block::new().el_with(Text::new("hello world!").el()),
        Block::new().el_with(Some(Text::new("hello world!").el())),
        Block::new().el_with(either! {
            match 0 {
                0 => Text::new("foo").el(),
                1 => Some(Text::new("foo").el()),
                _ => vec![Text::new("foo").el()],
            }
        }),
        Text::new("!").el(),
        Button(ButtonProps {
            label: "click me!".into(),
        })
        .el(),
        Counter().el().adapt(|state: &AppState| &state.count),
    ])
}

fn main() {
    let state = AppState {
        count: Data::from(0),
    };
    let env = AppEnv {};
    let root = app(&state);
    let mut widget_tree = WidgetTree::new(root, state, env);
    widget_tree.commit();
    println!("{:#?}", widget_tree);
}
