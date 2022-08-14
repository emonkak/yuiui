use either::Either;
use std::mem;

use crate::context::{EffectContext, RenderContext};
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::state::State;

use super::{CommitMode, ElementSeq, RenderStatus, WidgetNodeSeq};

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

impl<L, R, S> ElementSeq<S> for Either<L, R>
where
    L: ElementSeq<S>,
    R: ElementSeq<S>,
    S: State,
{
    type Store = EitherStore<L::Store, R::Store>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        match self {
            Either::Left(element) => EitherStore::new(Either::Left(element.render(state, context))),
            Either::Right(element) => {
                EitherStore::new(Either::Right(element.render(state, context)))
            }
        }
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        match (store.active.as_mut(), self) {
            (Either::Left(node), Either::Left(element)) => {
                if element.update(node, state, context) {
                    store.status = RenderStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (Either::Right(node), Either::Right(element)) => {
                if element.update(node, state, context) {
                    store.status = RenderStatus::Changed;
                    true
                } else {
                    false
                }
            }
            (Either::Left(_), Either::Right(element)) => {
                match store.staging.as_mut() {
                    Some(Either::Right(node)) => {
                        element.update(node, state, context);
                    }
                    None => {
                        store.staging = Some(Either::Right(element.render(state, context)));
                    }
                    _ => unreachable!(),
                };
                store.status = RenderStatus::Swapped;
                true
            }
            (Either::Right(_), Either::Left(element)) => {
                match store.staging.as_mut() {
                    Some(Either::Left(node)) => {
                        element.update(node, state, context);
                    }
                    None => {
                        store.staging = Some(Either::Left(element.render(state, context)));
                    }
                    _ => unreachable!(),
                }
                store.status = RenderStatus::Swapped;
                true
            }
        }
    }
}

impl<L, R, S> WidgetNodeSeq<S> for EitherStore<L, R>
where
    L: WidgetNodeSeq<S>,
    R: WidgetNodeSeq<S>,
    S: State,
{
    fn event_mask() -> EventMask {
        L::event_mask().merge(R::event_mask())
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        if self.status == RenderStatus::Swapped {
            match self.active.as_mut() {
                Either::Left(node) => node.commit(CommitMode::Unmount, state, context),
                Either::Right(node) => node.commit(CommitMode::Unmount, state, context),
            }
            mem::swap(&mut self.active, self.staging.as_mut().unwrap());
            if mode != CommitMode::Unmount {
                match self.active.as_mut() {
                    Either::Left(node) => node.commit(CommitMode::Mount, state, context),
                    Either::Right(node) => node.commit(CommitMode::Mount, state, context),
                }
            }
            self.status = RenderStatus::Unchanged;
        } else if self.status == RenderStatus::Changed || mode.is_propagatable() {
            match self.active.as_mut() {
                Either::Left(node) => node.commit(mode, state, context),
                Either::Right(node) => node.commit(mode, state, context),
            }
            self.status = RenderStatus::Unchanged;
        }
    }

    fn event<E: 'static>(
        &mut self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        match self.active.as_mut() {
            Either::Left(node) => node.event(event, state, context),
            Either::Right(node) => node.event(event, state, context),
        }
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        match self.active.as_mut() {
            Either::Left(node) => node.internal_event(event, state, context),
            Either::Right(node) => node.internal_event(event, state, context),
        }
    }
}
