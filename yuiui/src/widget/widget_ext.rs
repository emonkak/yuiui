use std::rc::Rc;

use super::{short_type_name_of, BoxedWidget, Widget, WidgetProxy};

pub trait WidgetExt<Own: ?Sized>: Widget<Own> {
    fn short_type_name(&self) -> &'static str {
        short_type_name_of(self.type_name())
    }

    fn into_boxed(self) -> BoxedWidget
    where
        Self: 'static + Sized + Widget<Self>,
    {
        Rc::new(WidgetProxy::new(self))
    }
}

impl<W: ?Sized + Widget<O>, O: ?Sized> WidgetExt<O> for W {}
