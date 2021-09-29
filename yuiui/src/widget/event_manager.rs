use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use yuiui_support::bit_flags::BitFlags;

use super::EventMask;

#[derive(Debug)]
pub struct EventManager<Listener> {
    listener_map: HashMap<EventMask, HashSet<Listener>>,
}

impl<Listener> EventManager<Listener> {
    pub fn new() -> Self {
        Self {
            listener_map: HashMap::new(),
        }
    }

    pub fn get_listerners(&self, event_mask: EventMask) -> Vec<Listener>
    where
        Listener: Copy
    {
        self.listener_map
            .get(&event_mask)
            .map_or(Vec::new(), |listeners| listeners.iter().copied().collect())
    }

    pub fn add_listener(&mut self, listener: Listener, event_masks: BitFlags<EventMask>)
    where
        Listener: Copy + Eq + Hash
    {
        for event_mask in event_masks.iter() {
            self.listener_map.entry(event_mask).or_default().insert(listener);
        }
    }

    pub fn remove_listener(&mut self, listener: Listener, event_masks: BitFlags<EventMask>)
    where
        Listener: Eq + Hash
    {
        for event_mask in event_masks.iter() {
            if let Some(listeners) = self.listener_map.get_mut(&event_mask) {
                listeners.remove(&listener);
            }
        }
    }
}
