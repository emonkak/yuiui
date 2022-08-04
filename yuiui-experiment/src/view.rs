use std::any::{self, Any};
use std::convert::Infallible;

use crate::children::Children;

pub trait View: 'static + AnyView {
    type Children: Children;
}

impl View for Infallible {
    type Children = ();
}

pub trait AnyView {
    fn name(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;
}

impl<T: View> AnyView for T {
    fn name(&self) -> &'static str {
        any::type_name::<T>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
