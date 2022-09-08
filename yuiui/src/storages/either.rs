use either::Either;
use std::mem;
use std::sync::Once;

use crate::context::{EffectContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::{EventMask, EventResult};
use crate::id::IdPath;
use crate::state::State;
use crate::traversable::{Monoid, Traversable};
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

impl<L, R, S, B> ElementSeq<S, B> for Either<L, R>
where
    L: ElementSeq<S, B>,
    R: ElementSeq<S, B>,
    S: State,
{
    type Storage = EitherStorage<L::Storage, R::Storage>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage {
        match self {
            Either::Left(element) => EitherStorage::new(Either::Left(
                element.render_children(state, backend, context),
            )),
            Either::Right(element) => EitherStorage::new(Either::Right(
                element.render_children(state, backend, context),
            )),
        }
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        match (&mut storage.active, self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.update_children(node, state, backend, context) {
                    storage.flags |= RenderFlags::UPDATED;
                    storage.flags -= RenderFlags::SWAPPED;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update_children(node, state, backend, context) {
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
                        element.update_children(node, state, backend, context);
                    }
                    None => {
                        storage.staging = Some(Either::Right(
                            element.render_children(state, backend, context),
                        ));
                    }
                    _ => unreachable!(),
                };
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match &mut storage.staging {
                    Some(Either::Left(node)) => {
                        element.update_children(node, state, backend, context);
                    }
                    None => {
                        storage.staging = Some(Either::Left(
                            element.render_children(state, backend, context),
                        ));
                    }
                    _ => unreachable!(),
                }
                storage.flags |= RenderFlags::SWAPPED;
                true
            }
        }
    }
}

impl<L, R, S, B> ViewNodeSeq<S, B> for EitherStorage<L, R>
where
    L: ViewNodeSeq<S, B>,
    R: ViewNodeSeq<S, B>,
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
        backend: &B,
        context: &mut EffectContext,
    ) -> EventResult<S> {
        let mut result = EventResult::nop();
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                result = result.combine(match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Unmount, state, backend, context),
                    Either::Right(node) => {
                        node.commit(CommitMode::Unmount, state, backend, context)
                    }
                });
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if mode != CommitMode::Unmount {
                result = result.combine(match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Mount, state, backend, context),
                    Either::Right(node) => node.commit(CommitMode::Mount, state, backend, context),
                });
            }
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagatable() {
            result = result.combine(match &mut self.active {
                Either::Left(node) => node.commit(mode, state, backend, context),
                Either::Right(node) => node.commit(mode, state, backend, context),
            });
        }
        self.flags = RenderFlags::COMMITED;
        result
    }
}

impl<L, R, Visitor, Context, Output, S, B> Traversable<Visitor, Context, Output, S, B>
    for EitherStorage<L, R>
where
    L: Traversable<Visitor, Context, Output, S, B>,
    R: Traversable<Visitor, Context, Output, S, B>,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> Output {
        match &mut self.active {
            Either::Left(node) => node.for_each(visitor, state, backend, context),
            Either::Right(node) => node.for_each(visitor, state, backend, context),
        }
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> Option<Output> {
        match &mut self.active {
            Either::Left(node) => node.search(id_path, visitor, state, backend, context),
            Either::Right(node) => node.search(id_path, visitor, state, backend, context),
        }
    }
}
