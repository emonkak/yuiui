use std::rc::Rc;

use crate::event::{EventHandler, EventManager, HandlerId};

#[derive(Debug)]
pub enum Lifecycle<Widget, Context> {
    WillMount,
    WillUpdate(Widget),
    WillUnmount,
    DidMount(Context),
    DidUpdate(Widget, Context),
    DidUnmount(Context),
}

pub struct LifecycleContext<'a, Handle> {
    pub(crate) event_manager: &'a mut EventManager<Handle>,
}

impl<Widget, Context> Lifecycle<Widget, Context> {
    pub fn map_widget<NewWidget, F: Fn(Widget) -> NewWidget>(
        self,
        f: F,
    ) -> Lifecycle<NewWidget, Context> {
        match self {
            Lifecycle::WillMount => Lifecycle::WillMount,
            Lifecycle::WillUpdate(widget) => Lifecycle::WillUpdate(f(widget)),
            Lifecycle::WillUnmount => Lifecycle::WillUnmount,
            Lifecycle::DidMount(context) => Lifecycle::DidMount(context),
            Lifecycle::DidUpdate(widget, context) => Lifecycle::DidUpdate(f(widget), context),
            Lifecycle::DidUnmount(context) => Lifecycle::DidUnmount(context),
        }
    }

    pub fn without_params(&self) -> Lifecycle<(), ()> {
        match self {
            Lifecycle::WillMount => Lifecycle::WillMount,
            Lifecycle::WillUpdate(_) => Lifecycle::WillUpdate(()),
            Lifecycle::WillUnmount => Lifecycle::WillUnmount,
            Lifecycle::DidMount(_) => Lifecycle::DidMount(()),
            Lifecycle::DidUpdate(_, _) => Lifecycle::DidUpdate((), ()),
            Lifecycle::DidUnmount(_) => Lifecycle::DidUnmount(()),
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
