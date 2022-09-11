use std::any::{Any, TypeId};
use std::collections::HashSet;

use crate::id::IdPathBuf;

pub trait Event<'event> {
    fn collect_types(type_ids: &mut Vec<TypeId>);

    fn from_any(event: &'event dyn Any) -> Option<Self>
    where
        Self: Sized;
}

impl<'event> Event<'event> for () {
    fn collect_types(_type_ids: &mut Vec<TypeId>) {}

    fn from_any(_event: &'event dyn Any) -> Option<Self> {
        None
    }
}

impl<'event, T: 'static> Event<'event> for &'event T {
    fn collect_types(type_ids: &mut Vec<TypeId>) {
        type_ids.push(TypeId::of::<T>())
    }

    fn from_any(event: &'event dyn Any) -> Option<Self> {
        event.downcast_ref()
    }
}

pub trait HasEvent<'event> {
    type Event: Event<'event>;
}

#[derive(Debug)]
pub struct EventMask {
    mask: Option<HashSet<TypeId>>,
}

impl EventMask {
    pub const fn new() -> Self {
        Self { mask: None }
    }

    pub fn contains(&self, type_id: &TypeId) -> bool {
        self.mask
            .as_ref()
            .map_or(false, |mask| mask.contains(type_id))
    }

    pub fn insert(&mut self, type_id: TypeId) {
        self.mask
            .get_or_insert_with(|| HashSet::new())
            .insert(type_id);
    }

    pub fn append(&mut self, other: &Self) {
        if let Some(mask) = &other.mask {
            if !mask.is_empty() {
                self.mask.get_or_insert_with(|| HashSet::new()).extend(mask);
            }
        }
    }
}

impl Extend<TypeId> for EventMask {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = TypeId>,
    {
        self.mask.get_or_insert_with(|| HashSet::new()).extend(iter);
    }
}

#[derive(Debug, Clone)]
pub enum EventDestination {
    Global,
    Local(IdPathBuf),
    Upward(IdPathBuf),
    Downward(IdPathBuf),
}

#[derive(Debug)]
pub enum Lifecycle<T> {
    Mount,
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
            Self::Update(value) => Lifecycle::Update(f(value)),
            Self::Unmount => Lifecycle::Unmount,
        }
    }
}
