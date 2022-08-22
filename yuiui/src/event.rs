use hlist::HNil;
use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::marker::PhantomData;

use crate::context::{EffectContext, IdPath};
use crate::effect::Effect;
use crate::state::State;
use crate::widget::Widget;

pub struct InternalEvent {
    pub id_path: IdPath,
    pub payload: Box<dyn Any>,
}

#[derive(Debug)]
pub struct EventMask {
    masks: HashSet<TypeId>,
}

impl EventMask {
    pub fn new() -> Self {
        Self {
            masks: HashSet::new(),
        }
    }

    pub fn contains(&self, type_id: &TypeId) -> bool {
        self.masks.contains(type_id)
    }

    pub fn add(&mut self, type_id: TypeId) {
        if type_id != TypeId::of::<()>() {
            self.masks.insert(type_id);
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.masks.extend(other.masks);
        self
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

pub struct EventListener<F, Event> {
    listener_fn: F,
    event_type: PhantomData<Event>,
}

impl<F, Event> EventListener<F, Event> {
    pub fn new<S, E>(listener_fn: F) -> Self
    where
        F: Fn(&Event, &S, &E, &mut EffectContext<S>),
        S: State,
        E: 'static,
    {
        Self {
            listener_fn,
            event_type: PhantomData,
        }
    }
}

impl<F, Event, S, E> Widget<S, E> for EventListener<F, Event>
where
    F: Fn(&Event, &S, &E) -> EventResult<S>,
    Event: 'static,
    S: State,
{
    type Children = HNil;

    type Event = Event;

    fn event(
        &mut self,
        event: &Self::Event,
        _children: &Self::Children,
        state: &S,
        env: &E,
    ) -> EventResult<S> {
        (self.listener_fn)(event, state, env)
    }
}
