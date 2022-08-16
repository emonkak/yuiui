use std::cmp::Ordering;
use std::ops::ControlFlow;

use crate::context::{EffectContext, RenderContext};
use crate::env::Env;
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::state::State;

use super::{CommitMode, ElementSeq, TraversableSeq, WidgetNodeSeq};

#[derive(Debug)]
pub struct VecStore<T> {
    active: Vec<T>,
    staging: Vec<T>,
    new_len: usize,
    dirty: bool,
}

impl<T> VecStore<T> {
    fn new(active: Vec<T>) -> Self {
        Self {
            staging: Vec::with_capacity(active.len()),
            new_len: active.len(),
            active,
            dirty: true,
        }
    }
}

impl<T, S, E> ElementSeq<S, E> for Vec<T>
where
    T: ElementSeq<S, E>,
    S: State,
    E: for<'a> Env<'a>,
{
    type Store = VecStore<T::Store>;

    fn render(
        self,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> Self::Store {
        VecStore::new(
            self.into_iter()
                .map(|element| element.render(state, env, context))
                .collect(),
        )
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> bool {
        let mut has_changed = false;

        store
            .staging
            .reserve_exact(self.len().saturating_sub(store.active.len()));
        store.new_len = self.len();

        for (i, element) in self.into_iter().enumerate() {
            if i < store.active.len() {
                let node = &mut store.active[i];
                has_changed |= element.update(node, state, env, context);
            } else {
                let j = i - store.active.len();
                if j < store.staging.len() {
                    let node = &mut store.staging[j];
                    has_changed |= element.update(node, state, env, context);
                } else {
                    let node = element.render(state, env, context);
                    store.staging.push(node);
                    has_changed = true;
                }
            }
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<T, S, E> WidgetNodeSeq<S, E> for VecStore<T>
where
    T: WidgetNodeSeq<S, E>,
    S: State,
    E: for<'a> Env<'a>,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut EffectContext<S>,
    ) {
        if self.dirty || mode.is_propagatable() {
            match self.new_len.cmp(&self.active.len()) {
                Ordering::Equal => {
                    for node in &mut self.active {
                        node.commit(mode, state, env, context);
                    }
                }
                Ordering::Less => {
                    // new_len < active_len
                    for node in &mut self.active[..self.new_len] {
                        node.commit(mode, state, env, context);
                    }
                    for mut node in self.active.drain(self.new_len..) {
                        node.commit(CommitMode::Unmount, state, env, context);
                        self.staging.push(node);
                    }
                }
                Ordering::Greater => {
                    // new_len > active_len
                    for node in &mut self.active {
                        node.commit(mode, state, env, context);
                    }
                    if mode != CommitMode::Unmount {
                        for i in 0..self.active.len() - self.new_len {
                            let mut node = self.staging.swap_remove(i);
                            node.commit(CommitMode::Mount, state, env, context);
                            self.active.push(node);
                        }
                    }
                }
            }
            self.dirty = false;
        }
    }

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let mut result = EventResult::Ignored;
        for node in &mut self.active {
            result = result.merge(node.event(event, state, env, context));
        }
        result
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        for node in &mut self.active {
            if node.internal_event(event, state, env, context) == EventResult::Captured {
                return EventResult::Captured;
            }
        }
        EventResult::Ignored
    }
}

impl<T, C> TraversableSeq<C> for VecStore<T>
where
    T: TraversableSeq<C>,
{
    fn for_each(&self, callback: &mut C) -> ControlFlow<()> {
        for node in &self.active {
            if let ControlFlow::Break(_) = node.for_each(callback) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}
