use hlist::{HCons, HList, HNil};

use crate::effect::EffectContext;
use crate::event::{CaptureState, EventMask, InternalEvent};
use crate::id::{IdContext, IdPath};
use crate::state::State;
use crate::widget_node::CommitMode;

use super::{ElementSeq, TraversableSeq, WidgetNodeSeq};

impl<S, E> ElementSeq<S, E> for HNil
where
    S: State,
{
    type Store = HNil;

    fn render(self, _state: &S, _env: &E, _context: &mut IdContext) -> Self::Store {
        HNil
    }

    fn update(
        self,
        _nodes: &mut Self::Store,
        _state: &S,
        _env: &E,
        _context: &mut IdContext,
    ) -> bool {
        false
    }
}

impl<S, E> WidgetNodeSeq<S, E> for HNil
where
    S: State,
{
    fn event_mask() -> EventMask {
        EventMask::new()
    }

    fn commit(&mut self, _mode: CommitMode, _state: &S, _env: &E, _context: &mut EffectContext<S>) {
    }

    fn event<Event: 'static>(
        &mut self,
        _event: &Event,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) -> CaptureState {
        CaptureState::Ignored
    }

    fn internal_event(
        &mut self,
        _event: &InternalEvent,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) -> CaptureState {
        CaptureState::Ignored
    }
}

impl<'a, V, S, E> TraversableSeq<V, S, E> for &'a HNil
where
    S: State,
{
    fn for_each(
        &mut self,
        _visitor: &mut V,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) {
    }

    fn search(
        &mut self,
        _id_path: &IdPath,
        _visitor: &mut V,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) -> bool {
        false
    }
}

impl<H, T, S, E> ElementSeq<S, E> for HCons<H, T>
where
    H: ElementSeq<S, E>,
    T: ElementSeq<S, E> + HList,
    T::Store: HList,
    S: State,
{
    type Store = HCons<H::Store, T::Store>;

    fn render(self, state: &S, env: &E, context: &mut IdContext) -> Self::Store {
        HCons {
            head: self.head.render(state, env, context),
            tail: self.tail.render(state, env, context),
        }
    }

    fn update(self, store: &mut Self::Store, state: &S, env: &E, context: &mut IdContext) -> bool {
        let mut has_changed = false;
        has_changed |= self.head.update(&mut store.head, state, env, context);
        has_changed |= self.tail.update(&mut store.tail, state, env, context);
        has_changed
    }
}

impl<H, T, S, E> WidgetNodeSeq<S, E> for HCons<H, T>
where
    H: WidgetNodeSeq<S, E>,
    T: WidgetNodeSeq<S, E> + HList,
    S: State,
{
    fn event_mask() -> EventMask {
        H::event_mask().merge(T::event_mask())
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        self.head.commit(mode, state, env, context);
        self.tail.commit(mode, state, env, context);
    }

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> CaptureState {
        self.head
            .event(event, state, env, context)
            .merge(self.tail.event(event, state, env, context))
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> CaptureState {
        if self.head.internal_event(event, state, env, context) == CaptureState::Captured {
            CaptureState::Captured
        } else {
            self.tail.internal_event(event, state, env, context)
        }
    }
}

impl<H, T, V, S, E> TraversableSeq<V, S, E> for HCons<H, T>
where
    H: TraversableSeq<V, S, E>,
    T: TraversableSeq<V, S, E> + HList,
    T: HList,
    S: State,
{
    fn for_each(&mut self, visitor: &mut V, state: &S, env: &E, context: &mut EffectContext<S>) {
        self.head.for_each(visitor, state, env, context);
        self.tail.for_each(visitor, state, env, context);
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        self.head.search(id_path, visitor, state, env, context)
            || self.tail.search(id_path, visitor, state, env, context)
    }
}
