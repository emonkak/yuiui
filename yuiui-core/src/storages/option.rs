use std::mem;

use crate::context::{CommitContext, RenderContext};
use crate::element::ElementSeq;
use crate::id::Id;
use crate::view_node::{CommitMode, Traversable, ViewNodeSeq};

use super::RenderFlags;

#[derive(Debug)]
pub struct OptionStorage<T> {
    active: Option<T>,
    staging: Option<T>,
    flags: RenderFlags,
}

impl<T> OptionStorage<T> {
    fn new(active: Option<T>) -> Self {
        Self {
            active,
            staging: None,
            flags: RenderFlags::NONE,
        }
    }
}

impl<T, S, M, E> ElementSeq<S, M, E> for Option<T>
where
    T: ElementSeq<S, M, E>,
{
    type Storage = OptionStorage<T::Storage>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        OptionStorage::new(self.map(|element| element.render_children(context)))
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        match (&mut storage.active, self) {
            (Some(node), Some(element)) => {
                if element.update_children(node, context) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = &mut storage.staging {
                    element.update_children(node, context);
                } else {
                    storage.staging = Some(element.render_children(context));
                }
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
            (Some(_), None) => {
                assert!(storage.staging.is_none());
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
            (None, None) => false,
        }
    }
}

impl<T, S, M, E> ViewNodeSeq<S, M, E> for OptionStorage<T>
where
    T: ViewNodeSeq<S, M, E>,
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

    fn commit(&mut self, mode: CommitMode, context: &mut CommitContext<S, M, E>) -> bool {
        let mut result = false;
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                if let Some(node) = &mut self.active {
                    result |= node.commit(CommitMode::Unmount, context);
                }
            }
            mem::swap(&mut self.active, &mut self.staging);
            if mode != CommitMode::Unmount {
                if let Some(node) = &mut self.active {
                    result |= node.commit(CommitMode::Mount, context);
                }
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagable() {
            if let Some(node) = &mut self.active {
                result |= node.commit(mode, context);
            }
        }
        self.flags = RenderFlags::COMMITED;
        result
    }

    fn gc(&mut self) {
        if let Some(node) = &mut self.active {
            node.gc();
        }
        if !self.flags.contains(RenderFlags::SWAPPED) {
            self.staging = None;
        }
    }
}

impl<T, Visitor, Context> Traversable<Visitor, Context> for OptionStorage<T>
where
    T: Traversable<Visitor, Context>,
{
    fn for_each(&mut self, visitor: &mut Visitor, context: &mut Context) {
        if let Some(node) = &mut self.active {
            node.for_each(visitor, context);
        }
    }

    fn for_id(&mut self, id: Id, visitor: &mut Visitor, context: &mut Context) -> bool {
        if let Some(node) = &mut self.active {
            node.for_id(id, visitor, context)
        } else {
            false
        }
    }
}
