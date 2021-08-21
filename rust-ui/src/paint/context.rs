use std::sync::Arc;

use crate::event::{EventHandler, EventManager, HandlerId};

#[derive(Debug)]
pub struct PaintContext<'a, Renderer> {
    event_manager: &'a mut EventManager<Renderer>,
}

impl<'a, Renderer> PaintContext<'a, Renderer> {
    pub fn new(event_manager: &'a mut EventManager<Renderer>) -> Self {
        Self { event_manager }
    }

    pub fn add_handler(&mut self, handler: Arc<dyn EventHandler<Renderer>>) -> HandlerId {
        self.event_manager.add(handler)
    }

    pub fn remove_handler(&mut self, handler_id: HandlerId) -> Arc<dyn EventHandler<Renderer>> {
        self.event_manager.remove(handler_id)
    }
}
