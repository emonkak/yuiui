use std::any::{Any, TypeId};
use std::fmt;

use crate::widget::message::{MessageSender, MessageSink};
use crate::widget::element::ElementId;

use super::event::EventType;

pub struct EventListener {
    pub type_id: TypeId,
    pub callback: EventCallback,
}

type EventCallback = Box<dyn Fn(&Box<dyn Any>, &MessageSender) + Send + Sync>;

impl fmt::Debug for EventListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventListener")
            .field("type_id", &self.type_id)
            .field("callback", &"{ .. }")
            .finish()
    }
}

impl EventListener {
    pub fn new<Message, EventType, ListenerFn>(
        element_id: ElementId,
        version: usize,
        event_type: EventType,
        listener_fn: ListenerFn,
    ) -> Self
    where
        Message: Send + Sync + 'static,
        EventType: self::EventType + 'static,
        EventType::Event: 'static,
        ListenerFn:
            Fn(&EventType::Event, MessageSink<Message, &MessageSender>) + Send + Sync + 'static,
    {
        Self {
            type_id: event_type.type_id(),
            callback: Box::new(move |event, message_sender| {
                listener_fn(
                    event.downcast_ref::<EventType::Event>().unwrap(),
                    MessageSink::new(element_id, version, message_sender),
                );
            }),
        }
    }

    pub fn dispatch(&self, event: &Box<dyn Any>, message_sender: &MessageSender) {
        (self.callback)(event, message_sender);
    }
}
