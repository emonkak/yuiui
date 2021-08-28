use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use crate::support::slot_vec::SlotVec;
use crate::widget::element::ElementId;

use super::listener::EventListener;

#[derive(Debug)]
pub struct EventManager {
    listeners: SlotVec<(ElementId, Arc<EventListener>)>,
    listeners_by_type: HashMap<TypeId, Vec<EventListenerId>>,
}

pub type EventListenerId = usize;

impl EventManager {
    pub fn new() -> Self {
        Self {
            listeners: SlotVec::new(),
            listeners_by_type: HashMap::new(),
        }
    }

    pub fn add_listener(
        &mut self,
        subscriber_id: ElementId,
        listener: Arc<EventListener>,
    ) -> EventListenerId {
        let type_id = listener.type_id;
        let listener_id = self.listeners.insert((subscriber_id, listener));
        self.listeners_by_type
            .entry(type_id)
            .or_default()
            .push(listener_id);
        listener_id
    }

    pub fn remove_listener(
        &mut self,
        listener_id: EventListenerId,
    ) -> (ElementId, Arc<EventListener>) {
        let (subscriber_id, listener) = self.listeners.remove(listener_id);
        let listener_ids = self.listeners_by_type.get_mut(&listener.type_id).unwrap();
        if let Some(index) = listener_ids.iter().position(|&id| id == listener_id) {
            listener_ids.swap_remove(index);
        }
        (subscriber_id, listener)
    }

    pub fn get_listeners(
        &self,
        type_id: TypeId,
    ) -> impl Iterator<Item = (ElementId, &Arc<EventListener>)> {
        let listeners = &self.listeners;
        self.listeners_by_type
            .get(&type_id)
            .map_or(&[] as &[EventListenerId], |listener_ids| {
                listener_ids.as_slice()
            })
            .iter()
            .map(move |listener_id| {
                let (subscriber_id, ref listener) = listeners[*listener_id];
                (subscriber_id, listener)
            })
    }

    // pub fn get_listeners_mut(&mut self, type_id: TypeId) -> impl Iterator<Item = &mut EventListener> {
    //     let listeners = &mut self.listeners;
    //     self.listeners_by_type.get(&type_id)
    //         .map_or(&[] as &[EventListenerId], |listener_ids| listener_ids.as_slice())
    //         .iter()
    //         .map(move |listener_id| {
    //             let listener_ptr = &mut listeners[*listener_id] as *mut EventListener;
    //             unsafe { listener_ptr.as_mut().unwrap() }
    //         })
    // }
}
