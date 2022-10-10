use std::any::Any;

use crate::id::IdPathBuf;

#[derive(Debug)]
pub enum Event {
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
    pub fn map<F, U>(self, f: F) -> Lifecycle<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Mount => Lifecycle::Mount,
            Self::Remount => Lifecycle::Mount,
            Self::Update(value) => Lifecycle::Update(f(value)),
            Self::Unmount => Lifecycle::Unmount,
        }
    }
}
