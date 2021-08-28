use std::sync::Arc;

use crate::event::{EventListener, EventListenerId, EventManager};
use crate::widget::element::ElementId;

#[derive(Debug)]
pub struct PaintContext<'a> {
    element_id: ElementId,
    event_manager: &'a mut EventManager,
}

impl<'a> PaintContext<'a> {
    pub fn new(element_id: ElementId, event_manager: &'a mut EventManager) -> Self {
        Self {
            element_id: element_id,
            event_manager,
        }
    }

    pub fn add_listener(&mut self, listener: Arc<EventListener>) -> EventListenerId {
        self.event_manager.add_listener(self.element_id, listener)
    }

    pub fn remove_listener(
        &mut self,
        listener_id: EventListenerId,
    ) -> (ElementId, Arc<EventListener>) {
        self.event_manager.remove_listener(listener_id)
    }
}
