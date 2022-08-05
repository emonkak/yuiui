use std::fmt::Debug;

use yuiui_experiment::*;

fn app() -> impl Element<
    View = impl View<
        Widget = impl Widget<Children = impl Debug> + Debug,
        Children = impl ElementSeq<Views = impl Debug>,
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
    let virtual_world = VirtualWorld::new(root);
    let real_world = virtual_world.realize();
    println!("{:#?}", virtual_world.view_pod());
    println!("{:#?}", real_world.widget_pod());
}
