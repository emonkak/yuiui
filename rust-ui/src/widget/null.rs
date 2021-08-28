use rust_ui_derive::WidgetMeta;

use super::widget::{Widget, WidgetMeta};

#[derive(WidgetMeta)]
pub struct Null;

impl<Renderer> Widget<Renderer> for Null {
    type State = ();
    type Message = ();
    type PaintObject = ();
}
