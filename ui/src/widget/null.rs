use std::any::Any;

use widget::widget::{Widget, WidgetMeta};

pub struct Null;

impl<Window> Widget<Window> for Null {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Default::default()
    }
}

impl WidgetMeta for Null {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
