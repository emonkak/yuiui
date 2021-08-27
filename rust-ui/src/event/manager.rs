use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use crate::support::slot_vec::SlotVec;

use super::handler::EventHandler;

pub struct EventManager {
    handlers: SlotVec<Arc<EventHandler>>,
    handlers_by_type: HashMap<TypeId, Vec<EventHandlerId>>,
}

pub type EventHandlerId = usize;

impl EventManager {
    pub fn new() -> Self {
        Self {
            handlers: SlotVec::new(),
            handlers_by_type: HashMap::new(),
        }
    }

    pub fn get(&self, type_id: &TypeId) -> impl Iterator<Item = &EventHandler> {
        self.handlers_by_type
            .get(type_id)
            .map_or(&[] as &[usize], |listener_ids| listener_ids.as_slice())
            .iter()
            .map(move |&handler_id| &*self.handlers[handler_id])
    }

    pub fn add(&mut self, handler: Arc<EventHandler>) -> EventHandlerId {
        let type_id = handler.subscribed_type();
        let handler_id = self.handlers.insert(handler);
        self.handlers_by_type
            .entry(type_id)
            .or_default()
            .push(handler_id);
        handler_id
    }

    pub fn remove(&mut self, handler_id: EventHandlerId) -> Arc<EventHandler> {
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
