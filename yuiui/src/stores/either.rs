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
pub struct EitherStore<L, R> {
    active: Either<L, R>,
    staging: Option<Either<L, R>>,
    flags: RenderFlags,
}

impl<L, R> EitherStore<L, R> {
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
    type Store = EitherStore<L::Store, R::Store>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        match self {
            Either::Left(element) => {
                EitherStore::new(Either::Left(element.render(state, env, context)))
            }
            Either::Right(element) => {
                EitherStore::new(Either::Right(element.render(state, env, context)))
            }
        }
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        match (&mut store.active, self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.update(node, state, env, context) {
                    store.flags |= RenderFlags::UPDATED;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update(node, state, env, context) {
                    store.flags |= RenderFlags::UPDATED;
                    true
                } else {
                    false
                }
            }
            (Either::Left(_), Either::Right(element)) => {
                match &mut store.staging {
                    Some(Either::Right(node)) => {
                        element.update(node, state, env, context);
                    }
                    None => {
                        store.staging = Some(Either::Right(element.render(state, env, context)));
                    }
                    _ => unreachable!(),
                };
                store.flags |= RenderFlags::SWAPPED;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match &mut store.staging {
                    Some(Either::Left(node)) => {
                        element.update(node, state, env, context);
                    }
                    None => {
                        store.staging = Some(Either::Left(element.render(state, env, context)));
                    }
                    _ => unreachable!(),
                }
                store.flags |= RenderFlags::SWAPPED;
                true
            }
        }
    }
}

impl<L, R, S, E> ViewNodeSeq<S, E> for EitherStore<L, R>
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

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        if self.flags.contains(RenderFlags::SWAPPED) {
            if self.flags.contains(RenderFlags::COMMITED) {
                match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Unmount, state, env, context),
                    Either::Right(node) => node.commit(CommitMode::Unmount, state, env, context),
                }
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if mode != CommitMode::Unmount {
                match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Mount, state, env, context),
                    Either::Right(node) => node.commit(CommitMode::Mount, state, env, context),
                }
            }
            self.flags = RenderFlags::COMMITED;
        } else if self.flags.contains(RenderFlags::UPDATED) || mode.is_propagatable() {
            match &mut self.active {
                Either::Left(node) => node.commit(mode, state, env, context),
                Either::Right(node) => node.commit(mode, state, env, context),
            }
            self.flags = RenderFlags::COMMITED;
        }
    }
}

impl<L, R, Visitor, Context, S, E> Traversable<Visitor, Context, S, E> for EitherStore<L, R>
where
    L: Traversable<Visitor, Context, S, E>,
    R: Traversable<Visitor, Context, S, E>,
    S: State,
{
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut Context) {
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
