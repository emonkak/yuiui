use std::any::{Any, TypeId};

#[derive(Debug)]
pub struct GenericEvent {
    pub type_id: TypeId,
    pub payload: Box<dyn Any>,
}

pub trait EventType: Send + Sync {
    type Event;

    fn of(event: impl Into<Self::Event>) -> GenericEvent
    where
        Self: 'static,
    {
        GenericEvent {
            type_id: TypeId::of::<Self>(),
            payload: Box::new(event.into()),
        }
    }

    fn downcast(event: &GenericEvent) -> Option<&Self::Event>
    where
        Self: 'static,
    {
        (&*event.payload).downcast_ref()
    }
}
