use std::cmp::Ordering;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::Id;
use crate::state::Store;
use crate::traversable::{Monoid, Traversable};
use crate::view_node::{CommitMode, ViewNodeSeq};

use super::binary_search_by;

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

impl<T, S, M, R, const N: usize> ElementSeq<S, M, R> for [T; N]
where
    T: ElementSeq<S, M, R>,
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

impl<'a, T, S, M, R, const N: usize> ViewNodeSeq<S, M, R> for ArrayStorage<T, N>
where
    T: ViewNodeSeq<S, M, R>,
{
    const SIZE_HINT: (usize, Option<usize>) = (N, Some(N));

    fn event_mask() -> &'static EventMask {
        T::event_mask()
    }

    fn len(&self) -> usize {
        match T::SIZE_HINT {
            (lower, Some(upper)) if lower == upper => lower * self.nodes.len(),
            _ => self.nodes.iter().map(|node| node.len()).sum(),
        }
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        if N > 0 {
            let first = self.nodes[0].id_range();
            let last = self.nodes[N - 1].id_range();
            match (first, last) {
                (Some((start, _)), Some((_, end))) => Some((start, end)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        let mut result = false;
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                result |= node.commit(mode, context, store, renderer);
            }
            self.dirty = false;
        }
        result
    }
}

impl<T, S, M, R, Visitor, Context, Output, const N: usize>
    Traversable<Visitor, Context, Output, S, M, R> for ArrayStorage<T, N>
where
    T: Traversable<Visitor, Context, Output, S, M, R> + ViewNodeSeq<S, M, R>,
    Output: Monoid,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Output {
        let mut result = Output::default();
        for node in &mut self.nodes {
            result = result.combine(node.for_each(visitor, context, store, renderer));
        }
        result
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Option<Output> {
        if T::SIZE_HINT.1.is_some() {
            if let Ok(index) = binary_search_by(&self.nodes, |node| {
                node.id_range().map(|(start, end)| {
                    if start < id {
                        Ordering::Less
                    } else if end > id {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                })
            }) {
                let node = &mut self.nodes[index];
                return node.for_id(id, visitor, context, store, renderer);
            }
        } else {
            for node in &mut self.nodes {
                if let Some(result) = node.for_id(id, visitor, context, store, renderer) {
                    return Some(result);
                }
            }
        }
        None
    }
}
