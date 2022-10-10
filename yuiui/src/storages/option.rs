use bit_flags::BitFlags;
use std::mem;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::id::Id;
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

    fn render_children(self, context: &mut RenderContext, state: &S) -> Self::Storage {
        let reserved_ids: Vec<Id> = T::Storage::SIZE_HINT
            .1
            .map(|upper| context.take_ids(upper).collect())
            .unwrap_or_default();
        context.preload_ids(&reserved_ids);
        OptionStorage::new(
            self.map(|element| element.render_children(context, state)),
            reserved_ids,
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
    ) -> bool {
        match (&mut storage.active, self) {
            (Some(node), Some(element)) => {
                if element.update_children(node, context, state) {
                    storage.flags |= RenderFlag::Updated;
                    storage.flags -= RenderFlag::Swapped;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = &mut storage.staging {
                    element.update_children(node, context, state);
                } else {
                    context.preload_ids(&storage.reserved_ids);
                    storage.staging = Some(element.render_children(context, state));
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
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        let mut result = false;
        if self.flags.contains(RenderFlag::Swapped) {
            if self.flags.contains(RenderFlag::Commited) {
                if let Some(node) = &mut self.active {
                    result |= node.commit(CommitMode::Unmount, context, store, renderer);
                }
            }
            mem::swap(&mut self.active, &mut self.staging);
            if mode != CommitMode::Unmount {
                if let Some(node) = &mut self.active {
                    result |= node.commit(CommitMode::Mount, context, store, renderer);
                }
            }
        } else if self.flags.contains(RenderFlag::Updated) || mode.is_propagatable() {
            if let Some(node) = &mut self.active {
                result |= node.commit(mode, context, store, renderer);
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

impl<T, Visitor, Context, Output, S, M, R> Traversable<Visitor, Context, Output, S, M, R>
    for OptionStorage<T>
where
    T: Traversable<Visitor, Context, Output, S, M, R>,
    Output: Default,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Output {
        if let Some(node) = &mut self.active {
            node.for_each(visitor, context, store, renderer)
        } else {
            Output::default()
        }
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Option<Output> {
        if let Some(node) = &mut self.active {
            node.for_id(id, visitor, context, store, renderer)
        } else {
            None
        }
    }
}
