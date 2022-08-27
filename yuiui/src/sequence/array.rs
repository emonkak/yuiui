use crate::effect::{EffectContext, EffectContextSeq, EffectContextVisitor};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::render::{IdPath, RenderContext, RenderContextSeq, RenderContextVisitor};
use crate::state::State;
use crate::widget_node::{CommitMode, WidgetNodeSeq};

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
    fn event_mask() -> EventMask {
        T::event_mask()
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

impl<T, S, E, const N: usize> RenderContextSeq<S, E> for ArrayStore<T, N>
where
    T: RenderContextSeq<S, E>,
    S: State,
{
    fn for_each<V: RenderContextVisitor>(
        &mut self,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) {
        for node in &mut self.nodes {
            node.for_each(visitor, state, env, context);
        }
    }

    fn search<V: RenderContextVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        for node in &mut self.nodes {
            if node.search(id_path, visitor, state, env, context) {
                return true;
            }
        }
        false
    }
}

impl<T, S, E, const N: usize> EffectContextSeq<S, E> for ArrayStore<T, N>
where
    T: EffectContextSeq<S, E>,
    S: State,
{
    fn for_each<V: EffectContextVisitor>(
        &mut self,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        for node in &mut self.nodes {
            node.for_each(visitor, state, env, context);
        }
    }

    fn search<V: EffectContextVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        for node in &mut self.nodes {
            if node.search(id_path, visitor, state, env, context) {
                return true;
            }
        }
        false
    }
}
