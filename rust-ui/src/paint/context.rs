use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::event::{EventHandler, EventManager, EventHandlerId};
use crate::widget::WidgetId;

pub struct PaintContext<'a> {
    widget_id: WidgetId,
    event_manager: &'a mut EventManager,
    update_sender: Sender<WidgetId>,
}

impl<'a> PaintContext<'a> {
    pub fn new(
        widget_id: WidgetId,
        event_manager: &'a mut EventManager,
        update_sender: Sender<WidgetId>,
    ) -> Self {
        Self { widget_id, event_manager, update_sender }
    }

    pub fn add_handler(&mut self, handler: Arc<EventHandler>) -> EventHandlerId {
        self.event_manager.add(handler)
    }

    pub fn remove_handler(&mut self, handler_id: EventHandlerId) -> Arc<EventHandler> {
        self.event_manager.remove(handler_id)
    }

    pub fn request_update(&self) {
        self.update_sender.send(self.widget_id).unwrap();
    }
}
