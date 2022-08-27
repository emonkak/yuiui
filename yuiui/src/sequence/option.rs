use std::mem;

use crate::context::{EffectContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::state::State;
use crate::widget_node::{CommitMode, WidgetNodeSeq};

use super::{RenderStatus, TraversableSeq};

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

    fn len(&self) -> usize {
        match &self.active {
            Some(node) => node.len(),
            None => 0,
        }
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
}

impl<T, Visitor, Context, S, E> TraversableSeq<Visitor, Context, S, E> for OptionStore<T>
where
    T: TraversableSeq<Visitor, Context, S, E>,
    S: State,
{
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut Context) {
        if let Some(node) = &mut self.active {
            node.for_each(visitor, state, env, context);
        }
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        if let Some(node) = &mut self.active {
            node.search(id_path, visitor, state, env, context)
        } else {
            false
        }
    }
}
