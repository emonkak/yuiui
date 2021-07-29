pub mod handler;
pub mod mouse;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::slot_vec::SlotVec;
use crate::widget::WidgetTree;

#[derive(Debug)]
pub struct EventManager<Handle> {
    handlers: SlotVec<Arc<dyn EventHandler<Handle> + Send + Sync>>,
    handlers_by_type: HashMap<TypeId, Vec<HandlerId>>,
}

pub struct EventContext {}

pub type HandlerId = usize;

pub trait EventType {
    type Event;
}

pub trait EventHandler<Handle> {
    fn dispatch(&self, tree: &WidgetTree<Handle>, event: &Box<dyn Any>, context: &mut EventContext);

    fn subscribed_type(&self) -> TypeId;

    fn as_ptr(&self) -> *const ();
}

impl<Handle> EventManager<Handle> {
    pub fn new() -> Self {
        Self {
            handlers: SlotVec::new(),
            handlers_by_type: HashMap::new(),
        }
    }

    pub fn get<T>(&self) -> impl Iterator<Item = &(dyn EventHandler<Handle> + Send + Sync)>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.handlers_by_type
            .get(&type_id)
            .map_or(&[] as &[usize], |listener_ids| listener_ids.as_slice())
            .iter()
            .map(move |&handler_id| &*self.handlers[handler_id])
    }

    pub fn add(&mut self, handler: Arc<dyn EventHandler<Handle> + Send + Sync>) -> HandlerId {
        let type_id = handler.subscribed_type();
        let handler_id = self.handlers.insert(handler);
        self.handlers_by_type
            .entry(type_id)
            .or_default()
            .push(handler_id);
        handler_id
    }

    pub fn remove(&mut self, handler_id: HandlerId) -> Arc<dyn EventHandler<Handle> + Send + Sync> {
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

impl<Handle> PartialEq for dyn EventHandler<Handle> + Send + Sync {
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl<Handle> Eq for dyn EventHandler<Handle> + Send + Sync {}

impl<Handle> fmt::Debug for dyn EventHandler<Handle> + Send + Sync {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_tuple("EventHandler")
            .field(&self.as_ptr())
            .finish()
    }
}
