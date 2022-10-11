use bit_flags::BitFlags;
use std::mem;

use crate::element::ElementSeq;
use crate::id::{Id, IdContext};
use crate::store::Store;
use crate::traversable::Traversable;
use crate::view_node::{CommitMode, ViewNodeSeq};

use super::RenderFlag;

#[derive(Debug)]
pub struct OptionStorage<T> {
    active: Option<T>,
    staging: Option<T>,
    flags: BitFlags<RenderFlag>,
    reserved_ids: Vec<Id>,
}

impl<T> OptionStorage<T> {
    fn new(active: Option<T>, reserved_ids: Vec<Id>) -> Self {
        Self {
            active,
            staging: None,
            flags: BitFlags::new(),
            reserved_ids,
        }
    }
}

impl<T, S, M, R> ElementSeq<S, M, R> for Option<T>
where
    T: ElementSeq<S, M, R>,
{
    type Storage = OptionStorage<T::Storage>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
        let reserved_ids: Vec<Id> = T::Storage::SIZE_HINT
            .1
            .map(|upper| id_context.take_ids(upper))
            .unwrap_or_default();
        id_context.preload_ids(&reserved_ids);
        OptionStorage::new(
            self.map(|element| element.render_children(id_context, state)),
            reserved_ids,
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        match (&mut storage.active, self) {
            (Some(node), Some(element)) => {
                if element.update_children(node, id_context, state) {
                    storage.flags |= RenderFlag::Updated;
                    storage.flags -= RenderFlag::Swapped;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = &mut storage.staging {
                    element.update_children(node, id_context, state);
                } else {
                    id_context.preload_ids(&storage.reserved_ids);
                    storage.staging = Some(element.render_children(id_context, state));
                }
                storage.flags |= RenderFlag::Swapped;
                true
            }
            (Some(_), None) => {
                assert!(storage.staging.is_none());
                storage.flags |= RenderFlag::Swapped;
                true
            }
            (None, None) => false,
        }
    }
}

impl<T, S, M, R> ViewNodeSeq<S, M, R> for OptionStorage<T>
where
    T: ViewNodeSeq<S, M, R>,
{
    const SIZE_HINT: (usize, Option<usize>) = {
        let (_, upper) = T::SIZE_HINT;
        (0, upper)
    };

    fn len(&self) -> usize {
        match &self.active {
            Some(node) => node.len(),
            None => 0,
        }
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        if self.reserved_ids.len() > 0 {
            Some((
                self.reserved_ids[0],
                self.reserved_ids[self.reserved_ids.len() - 1],
            ))
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
        renderer: &mut R,
    ) -> bool {
        let mut result = false;
        if self.flags.contains(RenderFlag::Swapped) {
            if self.flags.contains(RenderFlag::Commited) {
                if let Some(node) = &mut self.active {
                    result |=
                        node.commit(CommitMode::Unmount, id_context, store, messages, renderer);
                }
            }
            mem::swap(&mut self.active, &mut self.staging);
            if mode != CommitMode::Unmount {
                if let Some(node) = &mut self.active {
                    result |= node.commit(CommitMode::Mount, id_context, store, messages, renderer);
                }
            }
        } else if self.flags.contains(RenderFlag::Updated) || mode.is_propagatable() {
            if let Some(node) = &mut self.active {
                result |= node.commit(mode, id_context, store, messages, renderer);
            }
        }
        self.flags.set(RenderFlag::Commited);
        result
    }

    fn gc(&mut self) {
        if let Some(node) = &mut self.active {
            node.gc();
        }
        if !self.flags.contains(RenderFlag::Swapped) {
            self.staging = None;
        }
    }
}

impl<T, Visitor, Accumulator, S, M, R> Traversable<Visitor, Accumulator, S, M, R>
    for OptionStorage<T>
where
    T: Traversable<Visitor, Accumulator, S, M, R>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        accumulator: &mut Accumulator,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) {
        if let Some(node) = &mut self.active {
            node.for_each(visitor, accumulator, id_context, store, renderer);
        }
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        accumulator: &mut Accumulator,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        if let Some(node) = &mut self.active {
            node.for_id(id, visitor, accumulator, id_context, store, renderer)
        } else {
            false
        }
    }
}
