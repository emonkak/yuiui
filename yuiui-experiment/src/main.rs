use std::fmt::Debug;

use yuiui_experiment::*;

fn app() -> impl Element<
    View = impl View<
        Widget = impl Widget<Children = impl Debug> + Debug,
        Children = impl ElementSeq<VNodes = impl Debug>,
    > + Debug,
    Components = impl Debug,
> {
    view(
        Block::new(),
        (
            view(
                Block::new(),
                vec![view(Text::new("hello"), ()), view(Text::new("world"), ())],
            ),
            view(Text::new("!"), ()),
            component(Button::new("click me!")),
        ),
    )
}

fn main() {
    let root = app();
    let stage = Stage::new(root);
    println!("{:#?}", stage.v_node());
    println!("{:#?}", stage.ui_node());
}
