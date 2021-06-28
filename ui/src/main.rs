#[macro_use]
extern crate ui;

use ui::updater::UIUpdater;
use ui::geometrics::Size;
use ui::layout::BoxConstraints;
use ui::widget::flex::{Flex, FlexItem};
use ui::widget::null::Null;
use ui::widget::padding::Padding;
use ui::widget::widget::{WidgetMeta, Element};

fn render() -> Element<()> {
    element!(
        Padding::uniform(2.0) => {
            Flex::column() => {
                FlexItem::new(1.0).with_key(1) => { Null }
                FlexItem::new(1.0).with_key(2) => { Null }
                FlexItem::new(1.0).with_key(3) => { Null }
            }
        }
    )
}

fn render2() -> Element<()> {
    element!(
        Padding::uniform(2.0) => {
            Flex::column() => {
                FlexItem::new(1.0).with_key(2) => { Null }
                FlexItem::new(1.0).with_key(3) => { Null }
                FlexItem::new(1.0).with_key(1) => { Null }
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

    updater.force_update(render2());
    updater.render();
    updater.layout(BoxConstraints::tight(&Size {
        width: 640.0,
        height: 480.0,
    }));
    println!("{}", updater);
}
