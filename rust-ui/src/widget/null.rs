use rust_ui_derive::WidgetMeta;

use super::{Widget, WidgetMeta};

#[derive(WidgetMeta)]
pub struct Null;

impl<Handle> Widget<Handle> for Null {
    type State = ();
}
