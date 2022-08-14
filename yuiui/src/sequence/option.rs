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

impl<T, S> ElementSeq<S> for Option<T>
where
    T: ElementSeq<S>,
    S: State,
{
    type Store = OptionStore<T::Store>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        OptionStore::new(self.map(|element| element.render(state, context)))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        match (store.active.as_mut(), self) {
            (Some(node), Some(element)) => {
                if element.update(node, state, context) {
                    store.status = RenderStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = store.staging.as_mut() {
                    element.update(node, state, context);
                } else {
                    store.staging = Some(element.render(state, context));
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

impl<T, S> WidgetNodeSeq<S> for OptionStore<T>
where
    T: WidgetNodeSeq<S>,
    S: State,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        if self.status == RenderStatus::Swapped {
            if let Some(node) = self.active.as_mut() {
                node.commit(CommitMode::Unmount, state, context);
            }
            mem::swap(&mut self.active, &mut self.staging);
            if mode != CommitMode::Unmount {
                if let Some(node) = self.active.as_mut() {
                    node.commit(CommitMode::Mount, state, context);
                }
            }
            self.status = RenderStatus::Unchanged;
        } else if self.status == RenderStatus::Changed || mode.is_propagatable() {
            if let Some(node) = self.active.as_mut() {
                node.commit(mode, state, context);
            }
            self.status = RenderStatus::Unchanged;
        }
    }

    fn event<E: 'static>(
        &mut self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        if let Some(node) = self.active.as_mut() {
            node.event(event, state, context)
        } else {
            EventResult::Ignored
        }
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        if let Some(node) = self.active.as_mut() {
            node.internal_event(event, state, context)
        } else {
            EventResult::Ignored
        }
    }
}

impl<T, C> TraversableSeq<C> for OptionStore<T>
where
    T: TraversableSeq<C>,
{
    fn for_each(&self, callback: &mut C) -> ControlFlow<()> {
        if let Some(node) = self.active.as_ref() {
            if let ControlFlow::Break(_) = node.for_each(callback) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}
