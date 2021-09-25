use std::collections::HashMap;
use yuiui_support::slot_tree::NodeId;
use yuiui_support::bit_flags::BitFlags;

use crate::event::WindowEventMask;

#[derive(Debug)]
pub struct EventManager {
    listener_map: HashMap<WindowEventMask, Vec<NodeId>>,
    event_mask_map: HashMap<NodeId, BitFlags<WindowEventMask>>,
}

impl EventManager {
    pub fn get_listerners(&self, mask: WindowEventMask) -> &[NodeId] {
        self.listener_map.get(&mask).map_or(&[], |listeners| listeners.as_slice())
    }

    pub fn add_listener(&mut self, id: NodeId, masks: BitFlags<WindowEventMask>) {
        match self.event_mask_map.get_mut(&id) {
            Some(current_masks) => {
                let new_masks = *current_masks & (*current_masks ^ masks);
                for mask in new_masks.iter() {
                    self.listener_map.entry(mask)
                        .or_default()
                        .push(id);
                }
                *current_masks = *current_masks | masks;
            }
            None => {
                for mask in masks.iter() {
                    self.listener_map.entry(mask)
                        .or_default()
                        .push(id);
                }
                self.event_mask_map.insert(id, masks);
            }
        }
    }

    pub fn remove_listener(&mut self, id: NodeId, masks: BitFlags<WindowEventMask>) {
        if let Some(current_masks) = self.event_mask_map.get_mut(&id) {
            for mask in masks.iter() {
                if let Some(listeners) = self.listener_map.get_mut(&mask) {
                    if let Some(position) = listeners.iter().position(|listener| *listener == id) {
                        listeners.remove(position);
                    }
                }
            }
            let new_masks = *current_masks ^ masks;
            if new_masks.is_empty() {
                self.event_mask_map.remove(&id);
            } else {
                *current_masks = new_masks;
            }
        }
    }

    pub fn clear_listeners(&mut self, id: NodeId) {
        if let Some(masks) = self.event_mask_map.remove(&id) {
            for mask in masks.iter() {
                if let Some(listeners) = self.listener_map.get_mut(&mask) {
                    if let Some(position) = listeners.iter().position(|listener| *listener == id) {
                        listeners.remove(position);
                    }
                }
            }
        }
    }
}
