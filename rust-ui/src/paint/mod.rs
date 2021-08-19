pub mod layout;
pub mod tree;

use std::sync::Arc;

use crate::event::{EventHandler, EventManager, HandlerId};

pub struct LifecycleContext<'a, Renderer> {
    event_manager: &'a mut EventManager<Renderer>,
}

#[derive(Debug)]
pub enum Lifecycle<Widget, Children> {
    DidMount(Children),
    DidUpdate(Children, Widget, Children),
    DidUnmount(Children),
}

impl<'a, Renderer> LifecycleContext<'a, Renderer> {
    pub fn add_handler(&mut self, handler: Arc<dyn EventHandler<Renderer>>) -> HandlerId {
        self.event_manager.add(handler)
    }

    pub fn remove_handler(&mut self, handler_id: HandlerId) -> Arc<dyn EventHandler<Renderer>> {
        self.event_manager.remove(handler_id)
    }
}

impl<Widget, Children> Lifecycle<Widget, Children> {
    pub fn map<F, NewWidget>(self, f: F) -> Lifecycle<NewWidget, Children>
    where
        F: Fn(Widget) -> NewWidget,
    {
        match self {
            Lifecycle::DidMount(children) => Lifecycle::DidMount(children),
            Lifecycle::DidUpdate(children, new_widget, new_children) => {
                Lifecycle::DidUpdate(children, f(new_widget), new_children)
            }
            Lifecycle::DidUnmount(children) => Lifecycle::DidUnmount(children),
        }
    }

    pub fn without_params(&self) -> Lifecycle<(), ()> {
        match self {
            Lifecycle::DidMount(_) => Lifecycle::DidMount(()),
            Lifecycle::DidUpdate(_, _, _) => Lifecycle::DidUpdate((), (), ()),
            Lifecycle::DidUnmount(_) => Lifecycle::DidUnmount(()),
        }
    }
}
