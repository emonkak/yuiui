#[macro_use]
extern crate ui;

use ui::updater::UIUpdater;
use ui::geometrics::Size;
use ui::layout::BoxConstraints;
use ui::widget::flex::{FlexItem, Column};
use ui::widget::null::Null;
use ui::widget::padding::Padding;
use ui::widget::widget::{Element};

fn render() -> Element<()> {
    element!(
        Padding::uniform(2.0) => {
            Column::new() => {
                FlexItem::new(1.0) => { Null }
                FlexItem::new(1.0) => { Null }
                FlexItem::new(1.0) => { Null }
            }
        }
    )
}

fn render2() -> Element<()> {
    element!(
        Padding::uniform(2.0) => {
            Column::new() => {
                FlexItem::new(1.0) => {
                    Padding::uniform(2.0) => {
                        Column::new() => {
                            FlexItem::new(1.0) => { Null }
                            FlexItem::new(1.0) => { Null }
                        }
                    }
                }
                FlexItem::new(1.0) => {
                    Padding::uniform(2.0) => {
                        Column::new() => {
                            FlexItem::new(1.0) => { Null }
                            FlexItem::new(1.0) => { Null }
                        }
                    }
                }
            }
        }
    )
}

fn main() {
    let mut updater = UIUpdater::new(render());

    println!("{}", render());

    updater.render();
    updater.layout(BoxConstraints::tight(&Size {
        width: 640.0,
        height: 480.0,
    }));
    println!("{}", updater);

    updater.update(render2());
    updater.render();
    updater.layout(BoxConstraints::tight(&Size {
        width: 640.0,
        height: 480.0,
    }));
    println!("{}", updater);
}
