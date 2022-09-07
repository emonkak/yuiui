use crate::context::{EffectContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::state::State;
use crate::traversable::Traversable;
use crate::view_node::{CommitMode, ViewNodeSeq};

#[derive(Debug)]
pub struct ArrayStorage<T, const N: usize> {
    nodes: [T; N],
    dirty: bool,
}

impl<T, const N: usize> ArrayStorage<T, N> {
    fn new(nodes: [T; N]) -> Self {
        Self { nodes, dirty: true }
    }
}

impl<T, S, B, const N: usize> ElementSeq<S, B> for [T; N]
where
    T: ElementSeq<S, B>,
    S: State,
{
    type Storage = ArrayStorage<T::Storage, N>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage {
        ArrayStorage::new(self.map(|element| element.render_children(state, backend, context)))
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut storage.nodes[i];
            has_changed |= element.update_children(node, state, backend, context);
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<T, S, B, const N: usize> ViewNodeSeq<S, B> for ArrayStorage<T, N>
where
    T: ViewNodeSeq<S, B>,
    S: State,
{
    fn event_mask() -> &'static EventMask {
        T::event_mask()
    }

    fn len(&self) -> usize {
        N
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        backend: &B,
        context: &mut EffectContext<S>,
    ) -> bool {
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                node.commit(mode, state, backend, context);
            }
            self.dirty = false;
            true
        } else {
            false
        }
    }
}

impl<T, Visitor, Context, S, B, const N: usize> Traversable<Visitor, Context, S, B>
    for ArrayStorage<T, N>
where
    T: Traversable<Visitor, Context, S, B>,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> bool {
        let mut result = false;
        for node in &mut self.nodes {
            result |= node.for_each(visitor, state, backend, context);
        }
        result
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> bool {
        for node in &mut self.nodes {
            if node.search(id_path, visitor, state, backend, context) {
                return true;
            }
        }
        false
    }
}
