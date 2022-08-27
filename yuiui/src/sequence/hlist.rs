use hlist::{HCons, HList, HNil};
use std::sync::Once;

use crate::context::{EffectContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::state::State;
use crate::widget_node::{CommitMode, WidgetNodeSeq};

use super::TraversableSeq;

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
    fn event_mask() -> &'static EventMask {
        static MASK: EventMask = EventMask::new();
        &MASK
    }

    fn len(&self) -> usize {
        0
    }

    fn commit(&mut self, _mode: CommitMode, _state: &S, _env: &E, _context: &mut EffectContext<S>) {
    }
}

impl<Visitor, Context, S, E> TraversableSeq<Visitor, Context, S, E> for HNil
where
    S: State,
{
    fn for_each(&mut self, _visitor: &mut Visitor, _state: &S, _env: &E, _context: &mut Context) {}

    fn search(
        &mut self,
        _id_path: &IdPath,
        _visitor: &mut Visitor,
        _state: &S,
        _env: &E,
        _context: &mut Context,
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
    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let head_mask = H::event_mask();
            let tail_mask = T::event_mask();

            INIT.call_once(|| unsafe {
                EVENT_MASK.merge(head_mask);
                EVENT_MASK.merge(tail_mask);
            });
        }

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        self.head.len() + self.tail.len()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        self.head.commit(mode, state, env, context);
        self.tail.commit(mode, state, env, context);
    }
}

impl<H, T, Visitor, Context, S, E> TraversableSeq<Visitor, Context, S, E> for HCons<H, T>
where
    H: TraversableSeq<Visitor, Context, S, E>,
    T: TraversableSeq<Visitor, Context, S, E> + HList,
    S: State,
{
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut Context) {
        self.head.for_each(visitor, state, env, context);
        self.tail.for_each(visitor, state, env, context);
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        self.head.search(id_path, visitor, state, env, context)
            || self.tail.search(id_path, visitor, state, env, context)
    }
}
