use either_macro::either;
use std::fmt::Debug;

use yuiui_experiment::*;

#[derive(Debug)]
struct App {
    count: Immediate<i64>,
}

#[allow(dead_code)]
#[derive(Debug)]
enum AppMessage {
    Increment,
    Decrement,
}

impl State for App {
    type Message = AppMessage;

    fn reduce(&mut self, message: Self::Message) -> bool {
        match message {
            AppMessage::Increment => self.count.value += 1,
            AppMessage::Decrement => self.count.value -= 1,
        }
        true
    }
}

fn app(
    _state: &App,
) -> impl Element<
    App,
    View = impl View<App, Widget = impl Widget<App, Children = impl Debug> + Debug> + Debug,
    Components = impl Debug,
> {
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
        Button::new("click me!").el(),
        Counter.el().adapt(|state: &App| &state.count),
    ])
}

fn main() {
    let state = App {
        count: Immediate::from(0),
    };
    let root = app(&state);
    let mut stage = Stage::new(root, state);
    stage.commit();
    println!("{:#?}", stage);
}
