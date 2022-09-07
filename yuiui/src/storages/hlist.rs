use hlist::{HCons, HList, HNil};
use std::sync::Once;

use crate::context::{EffectContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::state::State;
use crate::traversable::Traversable;
use crate::view_node::{CommitMode, ViewNodeSeq};

impl<S, E> ElementSeq<S, E> for HNil
where
    S: State,
{
    type Storage = HNil;

    fn render(self, _state: &S, _env: &E, _context: &mut RenderContext) -> Self::Storage {
        HNil
    }

    fn update(
        self,
        _nodes: &mut Self::Storage,
        _state: &S,
        _env: &E,
        _context: &mut RenderContext,
    ) -> bool {
        false
    }
}

impl<S, E> ViewNodeSeq<S, E> for HNil
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

    fn commit(
        &mut self,
        _mode: CommitMode,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) -> bool {
        false
    }
}

impl<Visitor, Context, S, E> Traversable<Visitor, Context, S, E> for HNil
where
    S: State,
{
    fn for_each(
        &mut self,
        _visitor: &mut Visitor,
        _state: &S,
        _env: &E,
        _context: &mut Context,
    ) -> bool {
        false
    }

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
    T::Storage: HList,
    S: State,
{
    type Storage = HCons<H::Storage, T::Storage>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage {
        HCons {
            head: self.head.render(state, env, context),
            tail: self.tail.render(state, env, context),
        }
    }

    fn update(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let mut has_changed = false;
        has_changed |= self.head.update(&mut storage.head, state, env, context);
        has_changed |= self.tail.update(&mut storage.tail, state, env, context);
        has_changed
    }
}

impl<H, T, S, E> ViewNodeSeq<S, E> for HCons<H, T>
where
    H: ViewNodeSeq<S, E>,
    T: ViewNodeSeq<S, E> + HList,
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

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut has_changed = false;
        has_changed |= self.head.commit(mode, state, env, context);
        has_changed |= self.tail.commit(mode, state, env, context);
        has_changed
    }
}

impl<H, T, Visitor, Context, S, E> Traversable<Visitor, Context, S, E> for HCons<H, T>
where
    H: Traversable<Visitor, Context, S, E>,
    T: Traversable<Visitor, Context, S, E> + HList,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        let mut result = false;
        result |= self.head.for_each(visitor, state, env, context);
        result |= self.tail.for_each(visitor, state, env, context);
        result
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