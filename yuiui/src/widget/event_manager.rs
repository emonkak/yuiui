use std::collections::{HashMap, HashSet};
use yuiui_support::bit_flags::BitFlags;
use yuiui_support::slot_tree::NodeId;

use super::EventMask;

#[derive(Debug)]
pub struct EventManager {
    listener_map: HashMap<EventMask, HashSet<NodeId>>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            listener_map: HashMap::new(),
        }
    }

    pub fn get_listerners(&self, event_mask: EventMask) -> Vec<NodeId> {
        self.listener_map
            .get(&event_mask)
            .map_or(Vec::new(), |listeners| listeners.iter().copied().collect())
    }

    pub fn add_listener(&mut self, id: NodeId, event_masks: BitFlags<EventMask>) {
        for event_mask in event_masks.iter() {
            self.listener_map.entry(event_mask).or_default().insert(id);
        }
    }

    pub fn remove_listener(&mut self, id: NodeId, event_masks: BitFlags<EventMask>) {
        for event_mask in event_masks.iter() {
            if let Some(listeners) = self.listener_map.get_mut(&event_mask) {
                listeners.remove(&id);
            }
        }
    }
}
