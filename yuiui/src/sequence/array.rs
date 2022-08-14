use crate::context::{EffectContext, RenderContext};
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::state::State;

use super::{CommitMode, ElementSeq, WidgetNodeSeq};

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

impl<T, S, const N: usize> ElementSeq<S> for [T; N]
where
    T: ElementSeq<S>,
    S: State,
{
    type Store = ArrayStore<T::Store, N>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        ArrayStore::new(self.map(|element| element.render(state, context)))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut store.nodes[i];
            has_changed |= element.update(node, state, context);
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<T, S, const N: usize> WidgetNodeSeq<S> for ArrayStore<T, N>
where
    T: WidgetNodeSeq<S>,
    S: State,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                node.commit(mode, state, context);
            }
            self.dirty = false;
        }
    }

    fn event<E: 'static>(
        &mut self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let mut result = EventResult::Ignored;
        for node in &mut self.nodes {
            result = node.event(event, state, context);
        }
        result
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        for node in &mut self.nodes {
            if node.internal_event(event, state, context) == EventResult::Captured {
                return EventResult::Captured;
            }
        }
        EventResult::Ignored
    }
}
