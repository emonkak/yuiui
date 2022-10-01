use std::cmp::Ordering;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::Id;
use crate::state::Store;
use crate::traversable::{Monoid, Traversable};
use crate::view_node::{CommitMode, ViewNodeRange, ViewNodeSeq};

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
    T::Storage: ViewNodeRange,
{
    type Storage = ArrayStorage<T::Storage, N>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        ArrayStorage::new(self.map(|element| element.render_children(context, store)))
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut storage.nodes[i];
            has_changed |= element.update_children(node, context, store);
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<'a, T, S, M, B, const N: usize> ViewNodeSeq<S, M, B> for ArrayStorage<T, N>
where
    T: ViewNodeSeq<S, M, B> + ViewNodeRange,
{
    const IS_DYNAMIC: bool = T::IS_DYNAMIC;

    fn event_mask() -> &'static EventMask {
        T::event_mask()
    }

    fn len(&self) -> usize {
        self.nodes.iter().map(|node| node.len()).sum()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let mut result = false;
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                result |= node.commit(mode, context, store, backend);
            }
            self.dirty = false;
        }
        result
    }
}

impl<T, S, B, Visitor, Context, Output, const N: usize> Traversable<Visitor, Context, Output, S, B>
    for ArrayStorage<T, N>
where
    T: Traversable<Visitor, Context, Output, S, B> + ViewNodeRange,
    Output: Monoid,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        backend: &mut B,
    ) -> Output {
        let mut result = Output::default();
        for node in &mut self.nodes {
            result = result.combine(node.for_each(visitor, context, store, backend));
        }
        result
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        backend: &mut B,
    ) -> Option<Output> {
        if let Ok(index) = self.nodes.binary_search_by(|node| {
            let range = node.id_range();
            if range.start() < &id {
                Ordering::Less
            } else if range.end() > &id {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            let node = &mut self.nodes[index];
            node.for_id(id, visitor, context, store, backend)
        } else {
            None
        }
    }
}
