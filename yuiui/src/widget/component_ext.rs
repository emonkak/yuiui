use std::rc::Rc;

use super::{short_type_name_of, BoxedComponent, Component, ComponentProxy};

pub trait ComponentExt<Own: ?Sized>: Component<Own> {
    fn short_type_name(&self) -> &'static str {
        short_type_name_of(self.type_name())
    }

    fn into_boxed(self) -> BoxedComponent
    where
        Self: 'static + Sized + Component<Self>,
    {
        Rc::new(ComponentProxy::new(self))
    }
}

impl<C: ?Sized + Component<O>, O: ?Sized> ComponentExt<O> for C {}
