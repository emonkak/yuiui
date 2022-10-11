use bit_flags::BitFlags;
use either::Either;
use std::mem;

use crate::element::ElementSeq;
use crate::id::{Id, IdContext};
use crate::store::Store;
use crate::traversable::Traversable;
use crate::view_node::{CommitMode, ViewNodeSeq};

use super::RenderFlag;

#[derive(Debug)]
pub struct EitherStorage<L, R> {
    active: Either<L, R>,
    staging: Option<Either<L, R>>,
    flags: BitFlags<RenderFlag>,
    left_reserved_ids: Vec<Id>,
    right_reserved_ids: Vec<Id>,
}

impl<L, R> EitherStorage<L, R> {
    fn new(active: Either<L, R>, left_reserved_ids: Vec<Id>, right_reserved_ids: Vec<Id>) -> Self {
        Self {
            active,
            staging: None,
            flags: BitFlags::new(),
            left_reserved_ids,
            right_reserved_ids,
        }
    }
}

impl<L, R, S, M, Renderer> ElementSeq<S, M, Renderer> for Either<L, R>
where
    L: ElementSeq<S, M, Renderer>,
    R: ElementSeq<S, M, Renderer>,
{
    type Storage = EitherStorage<L::Storage, R::Storage>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
        let left_reserved_ids: Vec<Id> = L::Storage::SIZE_HINT
            .1
            .map(|upper| id_context.take_ids(upper))
            .unwrap_or_default();
        let right_reserved_ids: Vec<Id> = R::Storage::SIZE_HINT
            .1
            .map(|upper| id_context.take_ids(upper))
            .unwrap_or_default();
        match self {
            Either::Left(element) => {
                id_context.preload_ids(&left_reserved_ids);
                EitherStorage::new(
                    Either::Left(element.render_children(id_context, state)),
                    left_reserved_ids,
                    right_reserved_ids,
                )
            }
            Either::Right(element) => {
                id_context.preload_ids(&right_reserved_ids);
                EitherStorage::new(
                    Either::Right(element.render_children(id_context, state)),
                    left_reserved_ids,
                    right_reserved_ids,
                )
            }
        }
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        match (&mut storage.active, self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.update_children(node, id_context, state) {
                    storage.flags |= RenderFlag::Updated;
                    storage.flags -= RenderFlag::Swapped;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update_children(node, id_context, state) {
                    storage.flags |= RenderFlag::Updated;
                    storage.flags -= RenderFlag::Swapped;
                    true
                } else {
                    false
                }
            }
            (Either::Left(_), Either::Right(element)) => {
                match &mut storage.staging {
                    Some(Either::Right(node)) => {
                        element.update_children(node, id_context, state);
                    }
                    None => {
                        id_context.preload_ids(&storage.right_reserved_ids);
                        storage.staging =
                            Some(Either::Right(element.render_children(id_context, state)));
                    }
                    _ => unreachable!(),
                };
                storage.flags |= RenderFlag::Swapped;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match &mut storage.staging {
                    Some(Either::Left(node)) => {
                        element.update_children(node, id_context, state);
                    }
                    None => {
                        id_context.preload_ids(&storage.left_reserved_ids);
                        storage.staging =
                            Some(Either::Left(element.render_children(id_context, state)));
                    }
                    _ => unreachable!(),
                }
                storage.flags |= RenderFlag::Swapped;
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
    const SIZE_HINT: (usize, Option<usize>) = {
        let (left_lower, left_upper) = L::SIZE_HINT;
        let (right_lower, right_upper) = R::SIZE_HINT;
        let lower = if left_lower < right_lower {
            left_lower
        } else {
            right_lower
        };
        let upper = match (left_upper, right_upper) {
            (Some(x), Some(y)) => x.checked_add(y),
            _ => None,
        };
        (lower, upper)
    };

    fn len(&self) -> usize {
        match &self.active {
            Either::Left(node) => node.len(),
            Either::Right(node) => node.len(),
        }
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        match (
            !self.left_reserved_ids.is_empty(),
            !self.right_reserved_ids.is_empty(),
        ) {
            (true, true) => Some((
                self.left_reserved_ids[0],
                self.right_reserved_ids[self.right_reserved_ids.len() - 1],
            )),
            (true, false) => Some((
                self.left_reserved_ids[0],
                self.left_reserved_ids[self.left_reserved_ids.len() - 1],
            )),
            (false, true) => Some((
                self.right_reserved_ids[0],
                self.right_reserved_ids[self.right_reserved_ids.len() - 1],
            )),
            (false, false) => None,
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        renderer: &mut Renderer,
    ) -> bool {
        let mut result = false;
        if self.flags.contains(RenderFlag::Swapped) {
            if self.flags.contains(RenderFlag::Commited) {
                result |= match &mut self.active {
                    Either::Left(node) => {
                        node.commit(CommitMode::Unmount, id_context, store, messages, renderer)
                    }
                    Either::Right(node) => {
                        node.commit(CommitMode::Unmount, id_context, store, messages, renderer)
                    }
                };
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if mode != CommitMode::Unmount {
                result |= match &mut self.active {
                    Either::Left(node) => {
                        node.commit(CommitMode::Mount, id_context, store, messages, renderer)
                    }
                    Either::Right(node) => {
                        node.commit(CommitMode::Mount, id_context, store, messages, renderer)
                    }
                };
            }
        } else if self.flags.contains(RenderFlag::Updated) || mode.is_propagatable() {
            result |= match &mut self.active {
                Either::Left(node) => node.commit(mode, id_context, store, messages, renderer),
                Either::Right(node) => node.commit(mode, id_context, store, messages, renderer),
            };
        }
        self.flags.set(RenderFlag::Commited);
        result
    }

    fn gc(&mut self) {
        match &mut self.active {
            Either::Left(node) => node.gc(),
            Either::Right(node) => node.gc(),
        }
        if !self.flags.contains(RenderFlag::Swapped) {
            self.staging = None;
        }
    }
}

impl<L, R, Visitor, Accumulator, S, M, Renderer> Traversable<Visitor, Accumulator, S, M, Renderer>
    for EitherStorage<L, R>
where
    L: Traversable<Visitor, Accumulator, S, M, Renderer>,
    R: Traversable<Visitor, Accumulator, S, M, Renderer>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        accumulator: &mut Accumulator,
        id_context: &mut IdContext,
        store: &Store<S>,
    ) {
        match &mut self.active {
            Either::Left(node) => node.for_each(visitor, accumulator, id_context, store),
            Either::Right(node) => node.for_each(visitor, accumulator, id_context, store),
        }
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        accumulator: &mut Accumulator,
        id_context: &mut IdContext,
        store: &Store<S>,
    ) -> bool {
        match &mut self.active {
            Either::Left(node) => node.for_id(id, visitor, accumulator, id_context, store),
            Either::Right(node) => node.for_id(id, visitor, accumulator, id_context, store),
        }
    }
}
