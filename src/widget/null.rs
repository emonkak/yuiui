use std::any::Any;

use super::{Widget, WidgetMeta};

pub struct Null;

impl<Handle> Widget<Handle> for Null {
    type State = ();
}

impl WidgetMeta for Null {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
