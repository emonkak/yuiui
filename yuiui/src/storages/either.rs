use either::Either;
use std::mem;
use std::sync::Once;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::Id;
use crate::state::Store;
use crate::traversable::Traversable;
use crate::view_node::{CommitMode, ViewNodeSeq};

use super::RenderFlags;

#[derive(Debug)]
pub struct EitherStorage<L, R> {
    active: Either<L, R>,
    staging: Option<Either<L, R>>,
    flags: RenderFlags,
    reserved_ids: Vec<Id>,
}

impl<L, R> EitherStorage<L, R> {
    fn new(active: Either<L, R>, reserved_ids: Vec<Id>) -> Self {
        Self {
            active,
            staging: None,
            flags: RenderFlags::NONE,
            reserved_ids,
        }
    }
}

impl<L, R, S, M, Renderer> ElementSeq<S, M, Renderer> for Either<L, R>
where
    L: ElementSeq<S, M, Renderer>,
    R: ElementSeq<S, M, Renderer>,
{
    type Storage = EitherStorage<L::Storage, R::Storage>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        match self {
            Either::Left(element) => {
                let reserved_ids = R::Storage::SIZE_HINT
                    .1
                    .map(|upper| context.take_ids(upper).collect())
                    .unwrap_or_default();
                EitherStorage::new(
                    Either::Left(element.render_children(context, store)),
                    reserved_ids,
                )
            }
            Either::Right(element) => {
                let reserved_ids = L::Storage::SIZE_HINT
                    .1
                    .map(|upper| context.take_ids(upper).collect())
                    .unwrap_or_default();
                EitherStorage::new(
                    Either::Right(element.render_children(context, store)),
                    reserved_ids,
                )
            }
        }
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        match (&mut storage.active, self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.update_children(node, context, store) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update_children(node, context, store) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (Either::Left(_), Either::Right(element)) => {
                match &mut storage.staging {
                    Some(Either::Right(node)) => {
                        element.update_children(node, context, store);
                    }
                    None => {
                        context.reserve_ids(mem::take(&mut storage.reserved_ids));
                        storage.staging =
                            Some(Either::Right(element.render_children(context, store)));
                    }
                    _ => unreachable!(),
                };
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match &mut storage.staging {
                    Some(Either::Left(node)) => {
                        element.update_children(node, context, store);
                    }
                    None => {
                        context.reserve_ids(mem::take(&mut storage.reserved_ids));
                        storage.staging =
                            Some(Either::Left(element.render_children(context, store)));
                    }
                    _ => unreachable!(),
                }
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
        }
    }
}

impl<L, R, S, M, Renderer> ViewNodeSeq<S, M, Renderer> for EitherStorage<L, R>
where
    L: ViewNodeSeq<S, M, Renderer>,
    R: ViewNodeSeq<S, M, Renderer>,
{
    const IS_DYNAMIC: bool = true;

    const SIZE_HINT: (usize, Option<usize>) = {
        let (left_lower, left_upper) = L::SIZE_HINT;
        let (right_lower, right_upper) = R::SIZE_HINT;
        let lower = left_lower.saturating_add(right_lower);
        let upper = match (left_upper, right_upper) {
            (Some(x), Some(y)) => x.checked_add(y),
            _ => None,
        };
        (lower, upper)
    };

    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let left_mask = L::event_mask();
            let right_mask = R::event_mask();

            INIT.call_once(|| unsafe {
                if !left_mask.is_empty() {
                    EVENT_MASK.extend(left_mask);
                }
                if !right_mask.is_empty() {
                    EVENT_MASK.extend(right_mask);
                }
            });
        }

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        match &self.active {
            Either::Left(node) => node.len(),
            Either::Right(node) => node.len(),
        }
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        let active = match &self.active {
            Either::Left(node) => node.id_range(),
            Either::Right(node) => node.id_range(),
        };
        let staging = match &self.staging {
            Some(Either::Left(node)) => node.id_range(),
            Some(Either::Right(node)) => node.id_range(),
            None => {
                if self.reserved_ids.len() > 0 {
                    Some((
                        self.reserved_ids[0],
                        self.reserved_ids[self.reserved_ids.len() - 1],
                    ))
                } else {
                    None
                }
            }
        };
        match (active, staging) {
            (Some((x_start, x_end)), Some((y_start, y_end))) => {
                Some((x_start.min(y_start), x_end.max(y_end)))
            }
            _ => None,
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut Renderer,
    ) -> bool {
        let mut result = false;
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                result |= match &mut self.active {
                    Either::Left(node) => {
                        node.commit(CommitMode::Unmount, context, store, renderer)
                    }
                    Either::Right(node) => {
                        node.commit(CommitMode::Unmount, context, store, renderer)
                    }
                };
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if mode != CommitMode::Unmount {
                result |= match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Mount, context, store, renderer),
                    Either::Right(node) => node.commit(CommitMode::Mount, context, store, renderer),
                };
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagatable() {
            result |= match &mut self.active {
                Either::Left(node) => node.commit(mode, context, store, renderer),
                Either::Right(node) => node.commit(mode, context, store, renderer),
            };
        }
        self.flags = RenderFlags::COMMITED;
        result
    }
}

impl<L, R, Visitor, Context, Output, S, M, Renderer>
    Traversable<Visitor, Context, Output, S, M, Renderer> for EitherStorage<L, R>
where
    L: Traversable<Visitor, Context, Output, S, M, Renderer>,
    R: Traversable<Visitor, Context, Output, S, M, Renderer>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut Renderer,
    ) -> Output {
        match &mut self.active {
            Either::Left(node) => node.for_each(visitor, context, store, renderer),
            Either::Right(node) => node.for_each(visitor, context, store, renderer),
        }
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut Renderer,
    ) -> Option<Output> {
        match &mut self.active {
            Either::Left(node) => node.for_id(id, visitor, context, store, renderer),
            Either::Right(node) => node.for_id(id, visitor, context, store, renderer),
        }
    }
}
