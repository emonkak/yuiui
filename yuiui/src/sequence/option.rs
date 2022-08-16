use std::mem;
use std::ops::ControlFlow;

use crate::context::{EffectContext, RenderContext};
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::state::State;

use super::{CommitMode, ElementSeq, RenderStatus, TraversableSeq, WidgetNodeSeq};

#[derive(Debug)]
pub struct OptionStore<T> {
    active: Option<T>,
    staging: Option<T>,
    status: RenderStatus,
}

impl<T> OptionStore<T> {
    fn new(active: Option<T>) -> Self {
        Self {
            active,
            staging: None,
            status: RenderStatus::Unchanged,
        }
    }
}

impl<T, S, E> ElementSeq<S, E> for Option<T>
where
    T: ElementSeq<S, E>,
    S: State,
{
    type Store = OptionStore<T::Store>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        OptionStore::new(self.map(|element| element.render(state, env, context)))
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        match (&mut store.active, self) {
            (Some(node), Some(element)) => {
                if element.update(node, state, env, context) {
                    store.status = RenderStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = &mut store.staging {
                    element.update(node, state, env, context);
                } else {
                    store.staging = Some(element.render(state, env, context));
                }
                store.status = RenderStatus::Swapped;
                true
            }
            (Some(_), None) => {
                assert!(store.staging.is_none());
                store.status = RenderStatus::Swapped;
                true
            }
            (None, None) => false,
        }
    }
}

impl<T, S, E> WidgetNodeSeq<S, E> for OptionStore<T>
where
    T: WidgetNodeSeq<S, E>,
    S: State,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        if self.status == RenderStatus::Swapped {
            if let Some(node) = &mut self.active {
                node.commit(CommitMode::Unmount, state, env, context);
            }
            mem::swap(&mut self.active, &mut self.staging);
            if mode != CommitMode::Unmount {
                if let Some(node) = &mut self.active {
                    node.commit(CommitMode::Mount, state, env, context);
                }
            }
            self.status = RenderStatus::Unchanged;
        } else if self.status == RenderStatus::Changed || mode.is_propagatable() {
            if let Some(node) = &mut self.active {
                node.commit(mode, state, env, context);
            }
            self.status = RenderStatus::Unchanged;
        }
    }

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        if let Some(node) = &mut self.active {
            node.event(event, state, env, context)
        } else {
            EventResult::Ignored
        }
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        if let Some(node) = &mut self.active {
            node.internal_event(event, state, env, context)
        } else {
            EventResult::Ignored
        }
    }
}

impl<'a, T, C> TraversableSeq<C> for &'a OptionStore<T>
where
    &'a T: TraversableSeq<C>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        if let Some(node) = &self.active {
            if let ControlFlow::Break(_) = node.for_each(callback) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}

impl<'a, T, C> TraversableSeq<C> for &'a mut OptionStore<T>
where
    &'a mut T: TraversableSeq<C>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        if let Some(node) = &mut self.active {
            if let ControlFlow::Break(_) = node.for_each(callback) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}
