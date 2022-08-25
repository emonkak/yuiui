use std::any::{Any, TypeId};
use std::collections::HashSet;

use crate::effect::Effect;
use crate::id::IdPath;
use crate::state::State;

pub trait Event<'event> {
    fn allowed_types() -> Vec<TypeId>;

    fn from_any(value: &'event dyn Any) -> Option<Self>
    where
        Self: Sized;

    fn from_static<T: 'static>(value: &'event T) -> Option<Self>
    where
        Self: Sized;
}

impl<'event> Event<'event> for () {
    fn allowed_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn from_any(_value: &'event dyn Any) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    fn from_static<T: 'static>(_value: &'event T) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

pub struct InternalEvent {
    pub id_path: IdPath,
    pub payload: Box<dyn Any>,
}

#[derive(Debug)]
pub struct EventMask {
    mask: HashSet<TypeId>,
}

impl EventMask {
    pub fn new() -> Self {
        Self {
            mask: HashSet::new(),
        }
    }

    pub fn contains(&self, type_id: &TypeId) -> bool {
        self.mask.contains(type_id)
    }

    pub fn add(&mut self, type_id: TypeId) {
        self.mask.insert(type_id);
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.mask.extend(other.mask);
        self
    }
}

impl FromIterator<TypeId> for EventMask {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = TypeId>,
    {
        EventMask {
            mask: HashSet::from_iter(iter),
        }
    }
}

impl Extend<TypeId> for EventMask {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = TypeId>,
    {
        self.mask.extend(iter)
    }
}

#[must_use]
pub enum EventResult<S: State> {
    Nop,
    Effect(Effect<S>),
}

impl<S: State> EventResult<S> {
    pub fn map_effect<F, T>(self, f: F) -> EventResult<T>
    where
        F: FnOnce(Effect<S>) -> Effect<T>,
        T: State,
    {
        match self {
            Self::Nop => EventResult::Nop,
            Self::Effect(e) => EventResult::Effect(f(e)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CaptureState {
    Ignored,
    Captured,
}

impl CaptureState {
    pub fn merge(self, other: Self) -> Self {
        match self {
            Self::Ignored => other,
            Self::Captured => Self::Captured,
        }
    }
}
