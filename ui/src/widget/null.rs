use std::any::Any;

use widget::widget::{Element, Widget, WidgetMeta};

pub struct Null;

impl<Window> Widget<Window> for Null {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Default::default()
    }

    fn should_update(&self, _next_widget: &Self, _next_children: &[Element<Window>]) -> bool {
        false
    }
}

impl WidgetMeta for Null {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
