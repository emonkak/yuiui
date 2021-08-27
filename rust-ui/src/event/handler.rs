use std::any::{Any, TypeId};
use std::fmt;

pub struct EventHandler {
    type_id: TypeId,
    callback: Box<dyn Fn(&Box<dyn Any>) + Send + Sync>,
}

impl EventHandler {
    pub fn new<F, Event>(type_id: TypeId, callback: F) -> Self
    where
        F: Fn(&Event) + Send + Sync + 'static,
        Event: 'static,
    {
        Self {
            type_id,
            callback: Box::new(move |event| {
                callback(event.downcast_ref::<Event>().unwrap())
            }),
        }
    }

    pub fn subscribed_type(&self) -> TypeId {
        self.type_id
    }

    pub fn dispatch(&self, event: &Box<dyn Any>) {
        (self.callback)(event)
    }
}

impl fmt::Debug for EventHandler {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("EventHandler")
            .field("subscribed_type", &self.subscribed_type())
            .field("callback", &"{ .. }")
            .finish()
    }
}
