use std::ops::ControlFlow;

use crate::event::{CaptureState, EventContext, EventMask, InternalEvent};
use crate::id::IdContext;
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
{
    type Store = ArrayStore<T::Store, N>;

    fn render(self, state: &S, env: &E, context: &mut IdContext) -> Self::Store {
        ArrayStore::new(self.map(|element| element.render(state, env, context)))
    }

    fn update(self, store: &mut Self::Store, state: &S, env: &E, context: &mut IdContext) -> bool {
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
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EventContext<S>) {
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
        env: &E,
        context: &mut EventContext<S>,
    ) -> CaptureState {
        let mut capture_state = CaptureState::Ignored;
        for node in &mut self.nodes {
            capture_state = capture_state.merge(node.event(event, state, env, context));
        }
        capture_state
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EventContext<S>,
    ) -> CaptureState {
        for node in &mut self.nodes {
            if node.internal_event(event, state, env, context) == CaptureState::Captured {
                return CaptureState::Captured;
            }
        }
        CaptureState::Ignored
    }
}

impl<'a, T, C, const N: usize> TraversableSeq<C> for &'a ArrayStore<T, N>
where
    &'a T: TraversableSeq<C>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        for node in &self.nodes {
            if let ControlFlow::Break(_) = node.for_each(callback) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}

impl<'a, T, C, const N: usize> TraversableSeq<C> for &'a mut ArrayStore<T, N>
where
    &'a mut T: TraversableSeq<C>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        for node in &mut self.nodes {
            if let ControlFlow::Break(_) = node.for_each(callback) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}
