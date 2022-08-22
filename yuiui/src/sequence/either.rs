use either::Either;
use std::mem;
use std::ops::ControlFlow;

use crate::context::{EffectContext, RenderContext};
use crate::event::{CaptureState, EventMask, InternalEvent};
use crate::state::State;

use super::{CommitMode, ElementSeq, RenderStatus, TraversableSeq, WidgetNodeSeq};

#[derive(Debug)]
pub struct EitherStore<L, R> {
    active: Either<L, R>,
    staging: Option<Either<L, R>>,
    status: RenderStatus,
}

impl<L, R> EitherStore<L, R> {
    fn new(active: Either<L, R>) -> Self {
        Self {
            active,
            staging: None,
            status: RenderStatus::Unchanged,
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
                    store.status = RenderStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update(node, state, env, context) {
                    store.status = RenderStatus::Changed;
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
                store.status = RenderStatus::Swapped;
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
                store.status = RenderStatus::Swapped;
                true
            }
        }
    }
}

impl<L, R, S, E> WidgetNodeSeq<S, E> for EitherStore<L, R>
where
    L: WidgetNodeSeq<S, E>,
    R: WidgetNodeSeq<S, E>,
    S: State,
{
    fn event_mask() -> EventMask {
        L::event_mask().merge(R::event_mask())
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        if self.status == RenderStatus::Swapped {
            match &mut self.active {
                Either::Left(node) => node.commit(CommitMode::Unmount, state, env, context),
                Either::Right(node) => node.commit(CommitMode::Unmount, state, env, context),
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if mode != CommitMode::Unmount {
                match &mut self.active {
                    Either::Left(node) => node.commit(CommitMode::Mount, state, env, context),
                    Either::Right(node) => node.commit(CommitMode::Mount, state, env, context),
                }
            }
            self.status = RenderStatus::Unchanged;
        } else if self.status == RenderStatus::Changed || mode.is_propagatable() {
            match &mut self.active {
                Either::Left(node) => node.commit(mode, state, env, context),
                Either::Right(node) => node.commit(mode, state, env, context),
            }
            self.status = RenderStatus::Unchanged;
        }
    }

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> CaptureState {
        match &mut self.active {
            Either::Left(node) => node.event(event, state, env, context),
            Either::Right(node) => node.event(event, state, env, context),
        }
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> CaptureState {
        match &mut self.active {
            Either::Left(node) => node.internal_event(event, state, env, context),
            Either::Right(node) => node.internal_event(event, state, env, context),
        }
    }
}

impl<'a, L, R, C> TraversableSeq<C> for &'a EitherStore<L, R>
where
    &'a L: TraversableSeq<C>,
    &'a R: TraversableSeq<C>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        match &self.active {
            Either::Left(node) => node.for_each(callback),
            Either::Right(node) => node.for_each(callback),
        }
    }
}

impl<'a, L, R, C> TraversableSeq<C> for &'a mut EitherStore<L, R>
where
    &'a mut L: TraversableSeq<C>,
    &'a mut R: TraversableSeq<C>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        match &mut self.active {
            Either::Left(node) => node.for_each(callback),
            Either::Right(node) => node.for_each(callback),
        }
    }
}
