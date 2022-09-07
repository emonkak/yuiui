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

impl<T, S, E> ElementSeq<S, E> for Option<T>
where
    T: ElementSeq<S, E>,
    S: State,
{
    type Storage = OptionStorage<T::Storage>;

    fn render_children(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage {
        OptionStorage::new(self.map(|element| element.render_children(state, env, context)))
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        match (&mut storage.active, self) {
            (Some(node), Some(element)) => {
                if element.update_children(node, state, env, context) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = &mut storage.staging {
                    element.update_children(node, state, env, context);
                } else {
                    storage.staging = Some(element.render_children(state, env, context));
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

impl<T, S, E> ViewNodeSeq<S, E> for OptionStorage<T>
where
    T: ViewNodeSeq<S, E>,
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
        env: &E,
        context: &mut CommitContext<S>,
    ) -> bool {
        let mut has_changed = false;
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                if let Some(node) = &mut self.active {
                    has_changed |= node.commit(CommitMode::Unmount, state, env, context);
                }
            }
            mem::swap(&mut self.active, &mut self.staging);
            if mode != CommitMode::Unmount {
                if let Some(node) = &mut self.active {
                    has_changed |= node.commit(CommitMode::Mount, state, env, context);
                }
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagatable() {
            if let Some(node) = &mut self.active {
                has_changed |= node.commit(mode, state, env, context);
            }
        }
        self.flags = RenderFlags::COMMITED;
        has_changed
    }
}

impl<T, Visitor, Context, S, E> Traversable<Visitor, Context, S, E> for OptionStorage<T>
where
    T: Traversable<Visitor, Context, S, E>,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        if let Some(node) = &mut self.active {
            node.for_each(visitor, state, env, context)
        } else {
            false
        }
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        if let Some(node) = &mut self.active {
            node.search(id_path, visitor, state, env, context)
        } else {
            false
        }
    }
}
