use std::mem;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::Id;
use crate::state::Store;
use crate::traversable::Traversable;
use crate::view_node::{CommitMode, ViewNodeSeq};

use super::RenderFlags;

#[derive(Debug)]
pub struct OptionStorage<T> {
    active: Option<T>,
    staging: Option<T>,
    flags: RenderFlags,
    reserved_ids: Vec<Id>,
}

impl<T> OptionStorage<T> {
    fn new(active: Option<T>, reserved_ids: Vec<Id>) -> Self {
        Self {
            active,
            staging: None,
            flags: RenderFlags::NONE,
            reserved_ids,
        }
    }
}

impl<T, S, M, B> ElementSeq<S, M, B> for Option<T>
where
    T: ElementSeq<S, M, B>,
{
    type Storage = OptionStorage<T::Storage>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        let reserved_ids = if self.is_none() {
            T::Storage::SIZE_HINT
                .1
                .map(|upper| context.take_ids(upper).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        OptionStorage::new(
            self.map(|element| element.render_children(context, store)),
            reserved_ids,
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        match (&mut storage.active, self) {
            (Some(node), Some(element)) => {
                if element.update_children(node, context, store) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (None, Some(element)) => {
                if let Some(node) = &mut storage.staging {
                    element.update_children(node, context, store);
                } else {
                    context.reserve_ids(mem::take(&mut storage.reserved_ids));
                    storage.staging = Some(element.render_children(context, store));
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
    const IS_DYNAMIC: bool = true;

    const SIZE_HINT: (usize, Option<usize>) = {
        let (_, upper) = T::SIZE_HINT;
        (0, upper)
    };

    fn event_mask() -> &'static EventMask {
        T::event_mask()
    }

    fn len(&self) -> usize {
        match &self.active {
            Some(node) => node.len(),
            None => 0,
        }
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        self.active
            .as_ref()
            .or_else(|| self.staging.as_ref())
            .map_or_else(
                || {
                    if self.reserved_ids.len() > 0 {
                        Some((
                            self.reserved_ids[0],
                            self.reserved_ids[self.reserved_ids.len() - 1],
                        ))
                    } else {
                        None
                    }
                },
                |node| node.id_range(),
            )
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let mut result = false;
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                if let Some(node) = &mut self.active {
                    result |= node.commit(CommitMode::Unmount, context, store, backend);
                }
            }
            mem::swap(&mut self.active, &mut self.staging);
            if mode != CommitMode::Unmount {
                if let Some(node) = &mut self.active {
                    result |= node.commit(CommitMode::Mount, context, store, backend);
                }
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagatable() {
            if let Some(node) = &mut self.active {
                result |= node.commit(mode, context, store, backend);
            }
        }
        self.flags = RenderFlags::COMMITED;
        result
    }
}

impl<T, Visitor, Context, Output, S, M, B> Traversable<Visitor, Context, Output, S, M, B>
    for OptionStorage<T>
where
    T: Traversable<Visitor, Context, Output, S, M, B>,
    Output: Default,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        backend: &mut B,
    ) -> Output {
        if let Some(node) = &mut self.active {
            node.for_each(visitor, context, store, backend)
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
        backend: &mut B,
    ) -> Option<Output> {
        if let Some(node) = &mut self.active {
            node.for_id(id, visitor, context, store, backend)
        } else {
            None
        }
    }
}
