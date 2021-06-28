use std::any::Any;

use widget::widget::{Element, Widget, WidgetMeta};

pub struct Null;

impl<Window> Widget<Window> for Null {
    fn should_update(&self, _next_widget: &dyn Widget<Window>, _next_children: &[Element<Window>]) -> bool {
        false
    }
}

impl WidgetMeta for Null {
    fn as_any(&self) -> &dyn Any {
        self
    }
}