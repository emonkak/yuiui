use std::cmp::Ordering;

use crate::element::ElementSeq;
use crate::id::{Id, IdContext};
use crate::store::Store;
use crate::view_node::{CommitMode, Traversable, ViewNodeSeq};

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

impl<T, S, M, B, const N: usize> ElementSeq<S, M, B> for [T; N]
where
    T: ElementSeq<S, M, B>,
{
    type Storage = ArrayStorage<T::Storage, N>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
        ArrayStorage::new(self.map(|element| element.render_children(id_context, state)))
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut storage.nodes[i];
            has_changed |= element.update_children(node, id_context, state);
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<'a, T, S, M, B, const N: usize> ViewNodeSeq<S, M, B> for ArrayStorage<T, N>
where
    T: ViewNodeSeq<S, M, B>,
{
    const SIZE_HINT: (usize, Option<usize>) = (N, Some(N));

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
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        backend: &mut B,
    ) -> bool {
        let mut result = false;
        if self.dirty || mode.is_propagable() {
            for node in &mut self.nodes {
                result |= node.commit(mode, id_context, store, messages, backend);
            }
            self.dirty = false;
        }
        result
    }

    fn gc(&mut self) {
        if !T::IS_STATIC {
            for node in &mut self.nodes {
                node.gc();
            }
        }
    }
}

impl<T, Visitor, Context, S, M, B, const N: usize> Traversable<Visitor, Context, S, M, B>
    for ArrayStorage<T, N>
where
    T: Traversable<Visitor, Context, S, M, B> + ViewNodeSeq<S, M, B>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    ) {
        for node in &mut self.nodes {
            node.for_each(visitor, context, id_context);
        }
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    ) -> bool {
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
                return node.for_id(id, visitor, context, id_context);
            }
        } else {
            for node in &mut self.nodes {
                if node.for_id(id, visitor, context, id_context) {
                    return true;
                }
            }
        }
        false
    }
}
