use std::any::Any;

use crate::id::IdPathBuf;

pub trait EventTarget<'event> {
    type Event: Event<'event>;
}

pub trait Event<'event>: Sized {
    fn from_any(payload: &'event dyn Any) -> Option<Self>;
}

impl<'event> Event<'event> for () {
    #[inline]
    fn from_any(_payload: &'event dyn Any) -> Option<Self> {
        None
    }
}

impl<'event, T: 'static> Event<'event> for &'event T {
    #[inline]
    fn from_any(payload: &'event dyn Any) -> Option<Self> {
        payload.downcast_ref()
    }
}

#[derive(Debug)]
pub enum TransferableEvent {
    Forward(IdPathBuf, Box<dyn Any + Send>),
    Broadcast(Vec<IdPathBuf>, Box<dyn Any + Send>),
}

#[derive(Debug)]
pub enum Lifecycle<T> {
    Mount,
    Remount,
    Update(T),
    Unmount,
}

impl<T> Lifecycle<T> {
    #[inline]
    pub fn map<F, U>(self, f: F) -> Lifecycle<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Mount => Lifecycle::Mount,
            Self::Remount => Lifecycle::Remount,
            Self::Update(value) => Lifecycle::Update(f(value)),
            Self::Unmount => Lifecycle::Unmount,
        }
    }
}
