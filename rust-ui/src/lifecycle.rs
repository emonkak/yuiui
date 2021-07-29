use std::sync::Arc;

use crate::event::{EventHandler, EventManager, HandlerId};

#[derive(Debug)]
pub enum Lifecycle<Widget> {
    OnMount,
    OnUpdate(Widget),
    OnUnmount,
}

pub struct LifecycleContext<'a, Handle> {
    pub(crate) event_manager: &'a mut EventManager<Handle>,
}

impl<Widget> Lifecycle<Widget> {
    pub fn map<NewWidget, F: Fn(Widget) -> NewWidget>(self, f: F) -> Lifecycle<NewWidget> {
        match self {
            Lifecycle::OnMount => Lifecycle::OnMount,
            Lifecycle::OnUpdate(widget) => Lifecycle::OnUpdate(f(widget)),
            Lifecycle::OnUnmount => Lifecycle::OnUnmount,
        }
    }

    pub fn without_widget(&self) -> Lifecycle<()> {
        match self {
            Lifecycle::OnMount => Lifecycle::OnMount,
            Lifecycle::OnUpdate(_) => Lifecycle::OnUpdate(()),
            Lifecycle::OnUnmount => Lifecycle::OnUnmount,
        }
    }
}

impl<'a, Handle> LifecycleContext<'a, Handle> {
    pub fn add_handler(
        &mut self,
        handler: Arc<dyn EventHandler<Handle> + Send + Sync>,
    ) -> HandlerId {
        self.event_manager.add(handler)
    }

    pub fn remove_handler(
        &mut self,
        handler_id: HandlerId,
    ) -> Arc<dyn EventHandler<Handle> + Send + Sync> {
        self.event_manager.remove(handler_id)
    }
}
