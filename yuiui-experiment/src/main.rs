use yuiui_experiment::*;

fn app() -> Element<impl View, impl Component> {
    view(Block::new(), (
        view(Block::new(), vec![
            view(Text::new("hello"), ()),
            view(Text::new("world"), ()),
        ]),
        view(Text::new("!"), ()),
        component(Button::new("click me!"))
    ))
}

fn main() {
    let world = World::create(app());
    println!("{}", world.widget_tree);
    println!("{}", world.element_tree);
}
