use std::any::{self, Any};
use std::convert::Infallible;

use crate::children::Children;
use crate::widget::Widget;

pub trait View: 'static + AnyView {
    type Widget: Widget;

    type Children: Children;

    fn build(&self, children: &Self::Children) -> Self::Widget;

    fn rebuild(&self, children: &Self::Children, widget: &mut Self::Widget) -> bool {
        *widget = self.build(children);
        true
    }
}

impl View for Infallible {
    type Widget = ();

    type Children = ();

    fn build(&self, _children: &Self::Children) -> Self::Widget {
        ()
    }
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
