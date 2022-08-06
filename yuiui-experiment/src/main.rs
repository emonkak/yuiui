use std::fmt::Debug;

use yuiui_experiment::*;

fn app() -> impl Element<
    View = impl View<Widget = impl Widget<Children = impl Debug> + Debug> + Debug,
    Components = impl hlist::HList + Debug,
> {
    view(
        Block::new(),
        hlist![
            view(
                Block::new(),
                vec![
                    view(Text::new("hello"), hlist![]),
                    view(Text::new("world"), hlist![])
                ],
            ),
            view(
                Block::new(),
                Some(view(Text::new("hello world!"), hlist![]))
            ),
            view(Text::new("!"), hlist![]),
            component(Button::new("click me!")),
        ],
    )
}

fn main() {
    let root = app();
    let stage = Stage::new(root);
    println!("{:#?}", stage.node());
}
