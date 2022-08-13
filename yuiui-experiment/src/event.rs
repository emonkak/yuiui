use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;

use crate::context::EffectContext;
use crate::hlist::HNil;
use crate::state::State;
use crate::widget::Widget;

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

    pub fn contains<T: 'static>(&self) -> bool {
        self.masks.contains(&TypeId::of::<T>())
    }

    pub fn add<T: 'static>(&mut self) {
        let id = TypeId::of::<T>();
        if id != TypeId::of::<()>() {
            self.masks.insert(id);
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
