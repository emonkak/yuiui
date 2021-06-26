use std::any::Any;

use widget::widget::{Element, Widget, WidgetMaker};

pub struct Null;

impl<Window> Widget<Window> for Null {
    fn should_update(&self, _element: &Element<Window>) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WidgetMaker for Null {
}
