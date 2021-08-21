use rust_ui_derive::WidgetMeta;

use crate::graphics::Renderer;

use super::{Widget, WidgetMeta};

#[derive(WidgetMeta)]
pub struct Null;

impl<Renderer: self::Renderer> Widget<Renderer> for Null {
    type State = ();
}
