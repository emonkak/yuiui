use std::mem;

use crate::context::{CommitContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::state::State;
use crate::traversable::Traversable;
use crate::view_node::{CommitMode, ViewNodeSeq};

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

impl<T, S, B> ElementSeq<S, B> for Option<T>
where
    T: ElementSeq<S, B>,
    S: State,
{
    type Storage = OptionStorage<T::Storage>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage {
        OptionStorage::new(self.map(|element| element.render_children(state, backend, context)))
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        match (&mut storage.active, self) {
            (Some(node), Some(element)) => {
                if element.update_children(node, state, backend, context) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = &mut storage.staging {
                    element.update_children(node, state, backend, context);
                } else {
                    storage.staging = Some(element.render_children(state, backend, context));
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

impl<T, S, B> ViewNodeSeq<S, B> for OptionStorage<T>
where
    T: ViewNodeSeq<S, B>,
    S: State,
{
    fn event_mask() -> &'static EventMask {
        T::event_mask()
    }

    fn len(&self) -> usize {
        match &self.active {
            Some(node) => node.len(),
            None => 0,
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        let mut has_changed = false;
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                if let Some(node) = &mut self.active {
                    has_changed |= node.commit(CommitMode::Unmount, state, backend, context);
                }
            }
            mem::swap(&mut self.active, &mut self.staging);
            if mode != CommitMode::Unmount {
                if let Some(node) = &mut self.active {
                    has_changed |= node.commit(CommitMode::Mount, state, backend, context);
                }
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagatable() {
            if let Some(node) = &mut self.active {
                has_changed |= node.commit(mode, state, backend, context);
            }
        }
        self.flags = RenderFlags::COMMITED;
        has_changed
    }
}

impl<T, Visitor, Context, S, B> Traversable<Visitor, Context, S, B> for OptionStorage<T>
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
        if let Some(node) = &mut self.active {
            node.for_each(visitor, state, backend, context)
        } else {
            false
        }
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> bool {
        if let Some(node) = &mut self.active {
            node.search(id_path, visitor, state, backend, context)
        } else {
            false
        }
    }
}
