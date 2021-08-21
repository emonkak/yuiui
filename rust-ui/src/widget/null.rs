use rust_ui_derive::WidgetMeta;

use super::{Widget, WidgetMeta};

#[derive(WidgetMeta)]
pub struct Null;

impl<Renderer> Widget<Renderer> for Null {
    type State = ();
}
