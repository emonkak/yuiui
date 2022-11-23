use either::Either;
use std::mem;

use crate::context::{CommitContext, RenderContext};
use crate::element::ElementSeq;
use crate::id::Id;
use crate::view_node::{CommitMode, Traversable, ViewNodeSeq};

use super::RenderFlags;

#[derive(Debug)]
pub struct EitherStorage<L, R> {
    active: Either<L, R>,
    staging: Option<Either<L, R>>,
    flags: RenderFlags,
}

impl<L, R> EitherStorage<L, R> {
    fn new(active: Either<L, R>) -> Self {
        Self {
            active,
            staging: None,
            flags: RenderFlags::NONE,
        }
    }
}

impl<L, R, S, M, E> ElementSeq<S, M, E> for Either<L, R>
where
    L: ElementSeq<S, M, E>,
    R: ElementSeq<S, M, E>,
{
    type Storage = EitherStorage<L::Storage, R::Storage>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        match self {
            Either::Left(element) => {
                EitherStorage::new(Either::Left(element.render_children(context)))
            }
            Either::Right(element) => {
                EitherStorage::new(Either::Right(element.render_children(context)))
            }
        }
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        match (&mut storage.active, self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.update_children(node, context) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update_children(node, context) {
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
                        element.update_children(node, context);
                    }
                    None => {
                        storage.staging = Some(Either::Right(element.render_children(context)));
                    }
                    _ => unreachable!(),
                };
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match &mut storage.staging {
                    Some(Either::Left(node)) => {
                        element.update_children(node, context);
                    }
                    None => {
                        storage.staging = Some(Either::Left(element.render_children(context)));
                    }
                    _ => unreachable!(),
                }
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
        }
    }
}

impl<L, R, S, M, E> ViewNodeSeq<S, M, E> for EitherStorage<L, R>
where
    L: ViewNodeSeq<S, M, E>,
    R: ViewNodeSeq<S, M, E>,
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

    fn commit(&mut self, mode: CommitMode, context: &mut CommitContext<S, M, E>) -> bool {
        let mut result = false;
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                result |= match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Unmount, context),
                    Either::Right(node) => node.commit(CommitMode::Unmount, context),
                };
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if mode != CommitMode::Unmount {
                result |= match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Mount, context),
                    Either::Right(node) => node.commit(CommitMode::Mount, context),
                };
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagable() {
            result |= match &mut self.active {
                Either::Left(node) => node.commit(mode, context),
                Either::Right(node) => node.commit(mode, context),
            };
        }
        self.flags = RenderFlags::COMMITED;
        result
    }

    fn gc(&mut self) {
        match &mut self.active {
            Either::Left(node) => node.gc(),
            Either::Right(node) => node.gc(),
        }
        if !self.flags.contains(RenderFlags::SWAPPED) {
            self.staging = None;
        }
    }
}

impl<L, R, Visitor, Context> Traversable<Visitor, Context> for EitherStorage<L, R>
where
    L: Traversable<Visitor, Context>,
    R: Traversable<Visitor, Context>,
{
    fn for_each(&mut self, visitor: &mut Visitor, context: &mut Context) {
        match &mut self.active {
            Either::Left(node) => node.for_each(visitor, context),
            Either::Right(node) => node.for_each(visitor, context),
        }
    }

    fn for_id(&mut self, id: Id, visitor: &mut Visitor, context: &mut Context) -> bool {
        match &mut self.active {
            Either::Left(node) => node.for_id(id, visitor, context),
            Either::Right(node) => node.for_id(id, visitor, context),
        }
    }
}
