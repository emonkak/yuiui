use crate::context::{EffectContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::state::State;
use crate::widget_node::{CommitMode, WidgetNodeSeq};

use super::TraversableSeq;

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

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        ArrayStore::new(self.map(|element| element.render(state, env, context)))
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
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
{
    fn event_mask() -> &'static EventMask {
        T::event_mask()
    }

    fn len(&self) -> usize {
        N
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                node.commit(mode, state, env, context);
            }
            self.dirty = false;
        }
    }
}

impl<T, Visitor, Context, S, E, const N: usize> TraversableSeq<Visitor, Context, S, E>
    for ArrayStore<T, N>
where
    T: TraversableSeq<Visitor, Context, S, E>,
    S: State,
{
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut Context) {
        for node in &mut self.nodes {
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
        for node in &mut self.nodes {
            if node.search(id_path, visitor, state, env, context) {
                return true;
            }
        }
        false
    }
}
