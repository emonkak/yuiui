use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::sync::Arc;

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::effect::Effect;
use crate::state::State;

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
        if let Some(mask) = &self.mask {
            mask.contains(type_id)
        } else {
            false
        }
    }

    pub fn add(&mut self, type_id: TypeId) {
        self.mask
            .get_or_insert_with(|| HashSet::new())
            .insert(type_id);
    }

    pub fn add_all(&mut self, type_ids: &[TypeId]) {
        if !type_ids.is_empty() {
            self.mask
                .get_or_insert_with(|| HashSet::new())
                .extend(type_ids);
        }
    }

    pub fn merge(&mut self, other: &Self) {
        if let Some(mask) = &other.mask {
            if !mask.is_empty() {
                self.mask.get_or_insert_with(|| HashSet::new()).extend(mask);
            }
        }
    }
}

#[derive(Debug)]
pub enum Lifecycle<T> {
    Mounted,
    Remounted,
    Updated(T),
    Unmounted,
}

impl<T> Lifecycle<T> {
    pub fn map<F, U>(self, f: F) -> Lifecycle<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Mounted => Lifecycle::Mounted,
            Self::Remounted => Lifecycle::Remounted,
            Self::Updated(value) => Lifecycle::Updated(f(value)),
            Self::Unmounted => Lifecycle::Unmounted,
        }
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

    pub(crate) fn lift<F, NS>(self, f: &Arc<F>) -> EventResult<NS>
    where
        F: Fn(&NS) -> &S + Sync + Send + 'static,
        NS: State,
    {
        let effects = self
            .effects
            .into_iter()
            .map(|effect| effect.lift(f))
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

impl<S: State> From<Command<S>> for EventResult<S> {
    fn from(command: Command<S>) -> Self {
        EventResult {
            effects: vec![Effect::Command(command, None)],
        }
    }
}

impl<S: State> From<(Command<S>, CancellationToken)> for EventResult<S> {
    fn from((command, cancellation_token): (Command<S>, CancellationToken)) -> Self {
        EventResult {
            effects: vec![Effect::Command(command, Some(cancellation_token))],
        }
    }
}
