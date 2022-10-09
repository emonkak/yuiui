use std::any::{Any, TypeId};

pub trait EventTarget<'event> {
    type Event: Event<'event>;
}

pub trait Event<'event> {
    type Types: IntoIterator<Item = TypeId>;

    fn types() -> Self::Types;

    fn from_any(payload: &'event dyn Any) -> Option<Self>
    where
        Self: Sized;
}

impl<'event> Event<'event> for () {
    type Types = [TypeId; 0];

    fn types() -> Self::Types {
        []
    }

    fn from_any(_event: &'event dyn Any) -> Option<Self> {
        None
    }
}

impl<'event, T: 'static> Event<'event> for &'event T {
    type Types = [TypeId; 1];

    fn types() -> Self::Types {
        [TypeId::of::<T>()]
    }

    fn from_any(event: &'event dyn Any) -> Option<Self> {
        event.downcast_ref()
    }
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
