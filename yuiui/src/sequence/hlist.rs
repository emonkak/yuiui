use hlist::{HCons, HList, HNil};
use std::ops::ControlFlow;

use crate::context::{EffectContext, RenderContext};
use crate::event::{CaptureState, EventMask, InternalEvent};
use crate::state::State;

use super::{CommitMode, ElementSeq, TraversableSeq, WidgetNodeSeq};

impl<S, E> ElementSeq<S, E> for HNil
where
    S: State,
{
    type Store = HNil;

    fn render(self, _state: &S, _env: &E, _context: &mut RenderContext) -> Self::Store {
        HNil
    }

    fn update(
        self,
        _nodes: &mut Self::Store,
        _state: &S,
        _env: &E,
        _context: &mut RenderContext,
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

    fn commit(&mut self, _mode: CommitMode, _state: &S, _env: &E, _context: &mut EffectContext<S>) {}

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

impl<'a, C> TraversableSeq<C> for &'a HNil {
    fn for_each(self, _callback: &mut C) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

impl<'a, C> TraversableSeq<C> for &'a mut HNil {
    fn for_each(self, _callback: &mut C) -> ControlFlow<()> {
        ControlFlow::Continue(())
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

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        HCons {
            head: self.head.render(state, env, context),
            tail: self.tail.render(state, env, context),
        }
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
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

impl<'a, H, T, C> TraversableSeq<C> for &'a HCons<H, T>
where
    &'a H: TraversableSeq<C>,
    &'a T: TraversableSeq<C> + HList,
    T: HList,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        if let ControlFlow::Break(_) = self.head.for_each(callback) {
            ControlFlow::Break(())
        } else {
            self.tail.for_each(callback)
        }
    }
}

impl<'a, H, T, C> TraversableSeq<C> for &'a mut HCons<H, T>
where
    &'a mut H: TraversableSeq<C>,
    &'a mut T: TraversableSeq<C> + HList,
    T: HList,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        if let ControlFlow::Break(_) = self.head.for_each(callback) {
            ControlFlow::Break(())
        } else {
            self.tail.for_each(callback)
        }
    }
}
