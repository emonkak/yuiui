use std::any::Any;

use widget::widget::{Widget, WidgetMaker};

pub struct Null;

impl<Window> Widget<Window> for Null {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WidgetMaker for Null {
}
