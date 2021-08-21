pub mod handler;
pub mod mouse;
pub mod window;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::support::slot_vec::SlotVec;
use crate::support::tree::NodeId;
use crate::widget::tree::WidgetTree;

#[derive(Debug)]
pub struct EventManager<Renderer> {
    handlers: SlotVec<Arc<dyn EventHandler<Renderer>>>,
    handlers_by_type: HashMap<TypeId, Vec<HandlerId>>,
}

#[derive(Debug)]
pub struct GenericEvent {
    pub type_id: TypeId,
    pub payload: Box<dyn Any>,
}

pub type HandlerId = usize;

pub trait EventType: Send + Sync {
    type Event;

    fn of(event: impl Into<Self::Event>) -> GenericEvent
    where
        Self: 'static,
    {
        GenericEvent {
            type_id: TypeId::of::<Self>(),
            payload: Box::new(event.into()),
        }
    }

    fn downcast(event: &GenericEvent) -> Option<&Self::Event>
    where
        Self: 'static,
    {
        (&*event.payload).downcast_ref()
    }
}

pub trait EventHandler<Renderer>: Send + Sync {
    fn dispatch(
        &self,
        tree: &WidgetTree<Renderer>,
        event: &Box<dyn Any>,
        update_notifier: &Sender<NodeId>,
    );

    fn subscribed_type(&self) -> TypeId;

    fn as_ptr(&self) -> *const ();
}

impl<Renderer> EventManager<Renderer> {
    pub fn new() -> Self {
        Self {
            handlers: SlotVec::new(),
            handlers_by_type: HashMap::new(),
        }
    }

    pub fn get(&self, type_id: &TypeId) -> impl Iterator<Item = &(dyn EventHandler<Renderer>)> {
        self.handlers_by_type
            .get(type_id)
            .map_or(&[] as &[usize], |listener_ids| listener_ids.as_slice())
            .iter()
            .map(move |&handler_id| &*self.handlers[handler_id])
    }

    pub fn add(&mut self, handler: Arc<dyn EventHandler<Renderer>>) -> HandlerId {
        let type_id = handler.subscribed_type();
        let handler_id = self.handlers.insert(handler);
        self.handlers_by_type
            .entry(type_id)
            .or_default()
            .push(handler_id);
        handler_id
    }

    pub fn remove(&mut self, handler_id: HandlerId) -> Arc<dyn EventHandler<Renderer>> {
        let handler = self.handlers.remove(handler_id);
        let handler_ids = self
            .handlers_by_type
            .get_mut(&handler.subscribed_type())
            .unwrap();
        if let Some(index) = handler_ids.iter().position(|&id| id == handler_id) {
            handler_ids.swap_remove(index);
        }
        handler
    }
}

impl<Renderer> PartialEq for dyn EventHandler<Renderer> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl<Renderer> Eq for dyn EventHandler<Renderer> {}

impl<Renderer> fmt::Debug for dyn EventHandler<Renderer> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_tuple("EventHandler")
            .field(&self.as_ptr())
            .finish()
    }
}
