use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::sync::Arc;

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
    id_path: IdPath,
    payload: Box<dyn Any>,
}

impl InternalEvent {
    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn payload(&self) -> &dyn Any {
        self.payload.as_ref()
    }
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
pub struct EventResult<S: State> {
    effects: Vec<Effect<S>>,
}

impl<S: State> EventResult<S> {
    pub fn nop() -> Self {
        EventResult {
            effects: Vec::new(),
        }
    }

    pub fn into_effects(self) -> Vec<Effect<S>> {
        self.effects
    }

    pub(crate) fn lift<F, PS>(self, f: Arc<F>) -> EventResult<PS>
    where
        S: 'static,
        F: Fn(&PS) -> &S + Sync + Send + 'static,
        PS: State,
    {
        let effects = self
            .effects
            .into_iter()
            .map(move |effect| effect.lift(f.clone()))
            .collect();
        EventResult { effects }
    }
}

impl<S: State> From<Vec<Effect<S>>> for EventResult<S> {
    fn from(effects: Vec<Effect<S>>) -> Self {
        EventResult { effects }
    }
}

impl<S: State> From<Effect<S>> for EventResult<S> {
    fn from(effect: Effect<S>) -> Self {
        EventResult {
            effects: vec![effect],
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[must_use]
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
