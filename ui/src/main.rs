extern crate ui;

use ui::engine::UIEngine;
use ui::geometrics::Size;
use ui::layout::BoxConstraints;
use ui::widget::{Element, el};
use ui::widget::flex::{FlexItem, Column};
use ui::widget::null::Null;
use ui::widget::padding::Padding;

fn render() -> Element<(), ()> {
    el(Padding::uniform(2.0), [
        el(Column::new(), [
           el(FlexItem::new(1.0), [el(Null, [])]),
           el(FlexItem::new(1.0), [el(Null, [])]),
           el(FlexItem::new(1.0), [el(Null, [])]),
        ])
    ])
}

fn render2() -> Element<(), ()> {
    el(Padding::uniform(2.0), [
        el(Column::new(), [
            el(FlexItem::new(1.0), [
                el(Padding::uniform(2.0), [
                    el(Column::new(), [
                        el(FlexItem::new(1.0), [el(Null, [])]),
                        el(FlexItem::new(1.0), [el(Null, [])]),
                    ]),
                ]),
            ]),
            el(FlexItem::new(1.0), [
                el(Padding::uniform(2.0), [
                    el(Column::new(), [
                    el(FlexItem::new(1.0), [el(Null, [])]),
                    el(FlexItem::new(1.0), [el(Null, [])]),
                    ])
                ]),
            ]),
        ])
    ])
}

fn main() {
    let mut ui_state = UIEngine::new(render());

    println!("{}", render().to_string());

    ui_state.render();
    ui_state.layout(BoxConstraints::tight(&Size {
        width: 640.0,
        height: 480.0,
    }));
    println!("{}", ui_state.to_string());

    ui_state.update(render2());
    ui_state.render();
    ui_state.layout(BoxConstraints::tight(&Size {
        width: 640.0,
        height: 480.0,
    }));
    println!("{}", ui_state.to_string());
}
