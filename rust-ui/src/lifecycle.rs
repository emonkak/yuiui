use std::rc::Rc;

use crate::event::{EventHandler, EventManager, HandlerId};

pub enum Lifecycle<T> {
    WillMount,
    WillUpdate(T),
    WillUnmount,
    DidMount,
    DidUpdate(T),
    DidUnmount,
}

pub struct LifecycleContext<'a, Handle> {
    pub(crate) event_manager: &'a mut EventManager<Handle>,
}

impl<T> Lifecycle<T> {
    pub fn map<U, F: Fn(&T) -> U>(&self, f: F) -> Lifecycle<U> {
        match self {
            Lifecycle::WillMount => Lifecycle::WillMount,
            Lifecycle::WillUpdate(widget) => Lifecycle::WillUpdate(f(widget)),
            Lifecycle::WillUnmount => Lifecycle::WillUnmount,
            Lifecycle::DidMount => Lifecycle::DidMount,
            Lifecycle::DidUpdate(widget) => Lifecycle::DidUpdate(f(widget)),
            Lifecycle::DidUnmount => Lifecycle::DidUnmount,
        }
    }
}

impl<'a, Handle> LifecycleContext<'a, Handle> {
    pub fn add_handler(&mut self, handler: Rc<dyn EventHandler<Handle>>) -> HandlerId {
        self.event_manager.add(handler)
    }

    pub fn remove_handler(&mut self, handler_id: HandlerId) -> Rc<dyn EventHandler<Handle>> {
        self.event_manager.remove(handler_id)
    }
}
