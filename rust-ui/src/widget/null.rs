use rust_ui_derive::WidgetMeta;

use super::{Widget, WidgetMeta};

#[derive(WidgetMeta)]
pub struct Null;

impl<Painter> Widget<Painter> for Null {
    type State = ();
}
