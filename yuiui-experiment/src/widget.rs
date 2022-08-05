use std::any::{self, Any};

use crate::context::Id;

pub trait Widget: 'static + AnyWidget {
    type Children;
}

impl Widget for () {
    type Children = ();
}

pub trait AnyWidget {
    fn name(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Widget> AnyWidget for T {
    fn name(&self) -> &'static str {
        any::type_name::<T>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct WidgetPod<W: Widget> {
    pub(crate) id: Id,
    pub(crate) widget: W,
    pub(crate) children: W::Children,
}
