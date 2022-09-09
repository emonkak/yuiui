use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::traversable::{Monoid, Traversable};
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

impl<T, S, M, B, const N: usize> ElementSeq<S, M, B> for [T; N]
where
    T: ElementSeq<S, M, B>,
{
    type Storage = ArrayStorage<T::Storage, N>;

    const DEPTH: usize = T::DEPTH;

    fn render_children(self, context: &mut RenderContext, state: &S, backend: &B) -> Self::Storage {
        ArrayStorage::new(self.map(|element| element.render_children(context, state, backend)))
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut storage.nodes[i];
            has_changed |= element.update_children(node, context, state, backend);
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<T, S, M, B, const N: usize> ViewNodeSeq<S, M, B> for ArrayStorage<T, N>
where
    T: ViewNodeSeq<S, M, B>,
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
        context: &mut MessageContext<M>,
        state: &S,
        backend: &B,
    ) -> bool {
        let mut result = false;
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                result |= node.commit(mode, context, state, backend);
            }
            self.dirty = false;
        }
        result
    }
}

impl<T, Visitor, Context, Output, S, B, const N: usize> Traversable<Visitor, Context, Output, S, B>
    for ArrayStorage<T, N>
where
    T: Traversable<Visitor, Context, Output, S, B>,
    Output: Monoid,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Output {
        let mut result = Output::default();
        for node in &mut self.nodes {
            result = result.combine(node.for_each(visitor, context, state, backend));
        }
        result
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Option<Output> {
        for node in &mut self.nodes {
            if let Some(result) = node.search(id_path, visitor, context, state, backend) {
                return Some(result);
            }
        }
        None
    }
}
