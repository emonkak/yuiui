use std::any::{self, Any};

pub trait Widget: 'static {
}

impl Widget for () {
}

pub trait AnyWidget {
    fn name(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;
}

impl<T: Widget> AnyWidget for T {
    fn name(&self) -> &'static str {
        any::type_name::<T>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
