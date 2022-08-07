use either_macro::either;
use std::fmt::Debug;

use yuiui_experiment::*;

fn app() -> impl Element<
    View = impl View<Widget = impl Widget<Children = impl Debug> + Debug> + Debug,
    Components = impl hlist::HList + Debug,
> {
    Block::new().el_with(hlist![
        Block::new().el_with(vec![Text::new("hello").el(), Text::new("world").el()]),
        Block::new().el_with(Text::new("hello world!").el()),
        Block::new().el_with(Some(Text::new("hello world!").el())),
        Text::new("!").el(),
        Button::new("click me!").el(),
        Block::new().el_with(either! {
            match 0 {
                0 => Text::new("foo").el(),
                1 => Some(Text::new("foo").el()),
                _ => vec![Text::new("foo").el()],
            }
        })
    ])
}

fn main() {
    let root = app();
    let stage = Stage::new(root);
    println!("{:#?}", stage);
}
