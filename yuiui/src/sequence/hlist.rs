use crate::context::{EffectContext, RenderContext};
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::hlist::{HCons, HList, HNil};
use crate::state::State;

use super::{CommitMode, ElementSeq, WidgetNodeSeq};

impl<S: State> ElementSeq<S> for HNil {
    type Store = HNil;

    fn render(self, _state: &S, _context: &mut RenderContext) -> Self::Store {
        HNil
    }

    fn update(self, _nodes: &mut Self::Store, _state: &S, _context: &mut RenderContext) -> bool {
        false
    }
}

impl<S: State> WidgetNodeSeq<S> for HNil {
    fn event_mask() -> EventMask {
        EventMask::new()
    }

    fn commit(&mut self, _mode: CommitMode, _state: &S, _context: &mut EffectContext<S>) {}

    fn event<E: 'static>(
        &mut self,
        _event: &E,
        _state: &S,
        _context: &mut EffectContext<S>,
    ) -> EventResult {
        EventResult::Ignored
    }

    fn internal_event(
        &mut self,
        _event: &InternalEvent,
        _state: &S,
        _context: &mut EffectContext<S>,
    ) -> EventResult {
        EventResult::Ignored
    }
}

impl<H, T, S> ElementSeq<S> for HCons<H, T>
where
    H: ElementSeq<S>,
    T: ElementSeq<S> + HList,
    T::Store: HList,
    S: State,
{
    type Store = HCons<H::Store, T::Store>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store {
        HCons(self.0.render(state, context), self.1.render(state, context))
    }

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.update(&mut store.0, state, context);
        has_changed |= self.1.update(&mut store.1, state, context);
        has_changed
    }
}

impl<H, T, S> WidgetNodeSeq<S> for HCons<H, T>
where
    H: WidgetNodeSeq<S>,
    T: WidgetNodeSeq<S> + HList,
    S: State,
{
    fn event_mask() -> EventMask {
        H::event_mask().merge(T::event_mask())
    }

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        self.0.commit(mode, state, context);
        self.1.commit(mode, state, context);
    }

    fn event<E: 'static>(
        &mut self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        self.0
            .event(event, state, context)
            .merge(self.1.event(event, state, context))
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        if self.0.internal_event(event, state, context) == EventResult::Captured {
            EventResult::Captured
        } else {
            self.1.internal_event(event, state, context)
        }
    }
}
