use either::Either;
use std::mem;
use std::sync::Once;

use crate::context::{EffectContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::state::State;
use crate::traversable::Traversable;
use crate::view_node::{CommitMode, ViewNodeSeq};

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

impl<L, R, S, E> ElementSeq<S, E> for Either<L, R>
where
    L: ElementSeq<S, E>,
    R: ElementSeq<S, E>,
    S: State,
{
    type Storage = EitherStorage<L::Storage, R::Storage>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage {
        match self {
            Either::Left(element) => {
                EitherStorage::new(Either::Left(element.render(state, env, context)))
            }
            Either::Right(element) => {
                EitherStorage::new(Either::Right(element.render(state, env, context)))
            }
        }
    }

    fn update(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        match (&mut storage.active, self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.update(node, state, env, context) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update(node, state, env, context) {
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
                        element.update(node, state, env, context);
                    }
                    None => {
                        storage.staging = Some(Either::Right(element.render(state, env, context)));
                    }
                    _ => unreachable!(),
                };
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match &mut storage.staging {
                    Some(Either::Left(node)) => {
                        element.update(node, state, env, context);
                    }
                    None => {
                        storage.staging = Some(Either::Left(element.render(state, env, context)));
                    }
                    _ => unreachable!(),
                }
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
        }
    }
}

impl<L, R, S, E> ViewNodeSeq<S, E> for EitherStorage<L, R>
where
    L: ViewNodeSeq<S, E>,
    R: ViewNodeSeq<S, E>,
    S: State,
{
    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let left_mask = L::event_mask();
            let right_mask = R::event_mask();

            INIT.call_once(|| unsafe {
                EVENT_MASK.merge(left_mask);
                EVENT_MASK.merge(right_mask);
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

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut has_changed = false;
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                has_changed |= match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Unmount, state, env, context),
                    Either::Right(node) => node.commit(CommitMode::Unmount, state, env, context),
                };
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if mode != CommitMode::Unmount {
                has_changed |= match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Mount, state, env, context),
                    Either::Right(node) => node.commit(CommitMode::Mount, state, env, context),
                };
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagatable() {
            has_changed |= match &mut self.active {
                Either::Left(node) => node.commit(mode, state, env, context),
                Either::Right(node) => node.commit(mode, state, env, context),
            };
        }
        self.flags = RenderFlags::COMMITED;
        has_changed
    }
}

impl<L, R, Visitor, Context, S, E> Traversable<Visitor, Context, S, E> for EitherStorage<L, R>
where
    L: Traversable<Visitor, Context, S, E>,
    R: Traversable<Visitor, Context, S, E>,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        match &mut self.active {
            Either::Left(node) => node.for_each(visitor, state, env, context),
            Either::Right(node) => node.for_each(visitor, state, env, context),
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
        match &mut self.active {
            Either::Left(node) => node.search(id_path, visitor, state, env, context),
            Either::Right(node) => node.search(id_path, visitor, state, env, context),
        }
    }
}
