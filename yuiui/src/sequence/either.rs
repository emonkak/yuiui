use either::Either;
use std::mem;

use crate::effect::EffectContext;
use crate::event::{EventMask, InternalEvent};
use crate::id::{IdContext, IdPath};
use crate::state::State;
use crate::widget_node::CommitMode;

use super::{ElementSeq, RenderStatus, TraversableSeq, WidgetNodeSeq};

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

    fn render(self, state: &S, env: &E, context: &mut IdContext) -> Self::Store {
        match self {
            Either::Left(element) => {
                EitherStore::new(Either::Left(element.render(state, env, context)))
            }
            Either::Right(element) => {
                EitherStore::new(Either::Right(element.render(state, env, context)))
            }
        }
    }

    fn update(self, store: &mut Self::Store, state: &S, env: &E, context: &mut IdContext) -> bool {
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
    ) -> bool {
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
    ) -> bool {
        match &mut self.active {
            Either::Left(node) => node.internal_event(event, state, env, context),
            Either::Right(node) => node.internal_event(event, state, env, context),
        }
    }
}

impl<'a, L, R, V, S, E, C> TraversableSeq<V, S, E, C> for EitherStore<L, R>
where
    L: TraversableSeq<V, S, E, C>,
    R: TraversableSeq<V, S, E, C>,
    S: State,
{
    fn for_each(&mut self, visitor: &mut V, state: &S, env: &E, context: &mut C) {
        match &mut self.active {
            Either::Left(node) => node.for_each(visitor, state, env, context),
            Either::Right(node) => node.for_each(visitor, state, env, context),
        }
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut C,
    ) -> bool {
        match &mut self.active {
            Either::Left(node) => node.search(id_path, visitor, state, env, context),
            Either::Right(node) => node.search(id_path, visitor, state, env, context),
        }
    }
}
