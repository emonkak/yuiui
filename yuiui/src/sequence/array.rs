use std::ops::ControlFlow;

use crate::context::{EffectContext, RenderContext};
use crate::env::Env;
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::state::State;

use super::{CommitMode, ElementSeq, TraversableSeq, WidgetNodeSeq};

#[derive(Debug)]
pub struct ArrayStore<T, const N: usize> {
    nodes: [T; N],
    dirty: bool,
}

impl<T, const N: usize> ArrayStore<T, N> {
    fn new(nodes: [T; N]) -> Self {
        Self { nodes, dirty: true }
    }
}

impl<T, S, E, const N: usize> ElementSeq<S, E> for [T; N]
where
    T: ElementSeq<S, E>,
    S: State,
    E: for<'a> Env<'a>,
{
    type Store = ArrayStore<T::Store, N>;

    fn render(
        self,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> Self::Store {
        ArrayStore::new(self.map(|element| element.render(state, env, context)))
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut store.nodes[i];
            has_changed |= element.update(node, state, env, context);
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<T, S, E, const N: usize> WidgetNodeSeq<S, E> for ArrayStore<T, N>
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
            for node in &mut self.nodes {
                node.commit(mode, state, env, context);
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
        for node in &mut self.nodes {
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
        for node in &mut self.nodes {
            if node.internal_event(event, state, env, context) == EventResult::Captured {
                return EventResult::Captured;
            }
        }
        EventResult::Ignored
    }
}

impl<T, C, const N: usize> TraversableSeq<C> for ArrayStore<T, N>
where
    T: TraversableSeq<C>,
{
    fn for_each(&self, callback: &mut C) -> ControlFlow<()> {
        for node in &self.nodes {
            if let ControlFlow::Break(_) = node.for_each(callback) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}
