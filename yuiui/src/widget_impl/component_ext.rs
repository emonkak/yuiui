use crate::widget::Component;

pub trait ComponentExt<Own: ?Sized>: Component<Own> {}

impl<C: ?Sized + Component<O>, O: ?Sized> ComponentExt<O> for C {}
