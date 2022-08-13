use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::marker::PhantomData;

use crate::context::{EffectContext, IdPath};
use crate::hlist::HNil;
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

pub struct EventListener<F, E> {
    listener_fn: F,
    event_type: PhantomData<E>,
}

impl<F, E> EventListener<F, E> {
    pub fn new<S>(listener_fn: F) -> Self
    where
        S: State,
        F: Fn(&E, &S, &mut EffectContext<S>),
        E: 'static,
    {
        Self {
            listener_fn,
            event_type: PhantomData,
        }
    }
}

impl<S, F, E> Widget<S> for EventListener<F, E>
where
    S: State,
    F: Fn(&E, &S, &mut EffectContext<S>),
    E: 'static,
{
    type Children = HNil;

    type Event = E;

    fn event(&self, event: &Self::Event, state: &S, context: &mut EffectContext<S>) {
        (self.listener_fn)(event, state, context)
    }
}
