use std::mem;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
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

impl<T, S, M, B> ElementSeq<S, M, B> for Option<T>
where
    T: ElementSeq<S, M, B>,
{
    type Storage = OptionStorage<T::Storage>;

    const DEPTH: usize = T::DEPTH;

    fn render_children(self, context: &mut RenderContext, state: &S, backend: &B) -> Self::Storage {
        OptionStorage::new(self.map(|element| element.render_children(context, state, backend)))
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        match (&mut storage.active, self) {
            (Some(node), Some(element)) => {
                if element.update_children(node, context, state, backend) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = &mut storage.staging {
                    element.update_children(node, context, state, backend);
                } else {
                    storage.staging = Some(element.render_children(context, state, backend));
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

impl<T, S, M, B> ViewNodeSeq<S, M, B> for OptionStorage<T>
where
    T: ViewNodeSeq<S, M, B>,
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
        context: &mut MessageContext<M>,
        state: &S,
        backend: &B,
    ) -> bool {
        let mut result = false;
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                if let Some(node) = &mut self.active {
                    result |= node.commit(CommitMode::Unmount, context, state, backend);
                }
            }
            mem::swap(&mut self.active, &mut self.staging);
            if mode != CommitMode::Unmount {
                if let Some(node) = &mut self.active {
                    result |= node.commit(CommitMode::Mount, context, state, backend);
                }
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagatable() {
            if let Some(node) = &mut self.active {
                result |= node.commit(mode, context, state, backend);
            }
        }
        self.flags = RenderFlags::COMMITED;
        result
    }
}

impl<T, Visitor, Context, Output, S, B> Traversable<Visitor, Context, Output, S, B>
    for OptionStorage<T>
where
    T: Traversable<Visitor, Context, Output, S, B>,
    Output: Default,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Output {
        if let Some(node) = &mut self.active {
            node.for_each(visitor, context, state, backend)
        } else {
            Output::default()
        }
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Option<Output> {
        if let Some(node) = &mut self.active {
            node.search(id_path, visitor, context, state, backend)
        } else {
            None
        }
    }
}
