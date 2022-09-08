use hlist::{HCons, HList, HNil};
use std::sync::Once;

use crate::context::{EffectContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::state::State;
use crate::traversable::{Monoid, Traversable};
use crate::view_node::{CommitMode, ViewNodeSeq};

impl<S, B> ElementSeq<S, B> for HNil
where
    S: State,
{
    type Storage = HNil;

    fn render_children(
        self,
        _state: &S,
        _backend: &B,
        _context: &mut RenderContext,
    ) -> Self::Storage {
        HNil
    }

    fn update_children(
        self,
        _nodes: &mut Self::Storage,
        _state: &S,
        _backend: &B,
        _context: &mut RenderContext,
    ) -> bool {
        false
    }
}

impl<S, B> ViewNodeSeq<S, B> for HNil
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
        _backend: &B,
        _context: &mut EffectContext<S>,
    ) -> bool {
        false
    }
}

impl<Visitor, Context, Output, S, B> Traversable<Visitor, Context, Output, S, B> for HNil
where
    Output: Default,
    S: State,
{
    fn for_each(
        &mut self,
        _visitor: &mut Visitor,
        _state: &S,
        _backend: &B,
        _context: &mut Context,
    ) -> Output {
        Output::default()
    }

    fn search(
        &mut self,
        _id_path: &IdPath,
        _visitor: &mut Visitor,
        _state: &S,
        _backend: &B,
        _context: &mut Context,
    ) -> Option<Output> {
        None
    }
}

impl<H, T, S, B> ElementSeq<S, B> for HCons<H, T>
where
    H: ElementSeq<S, B>,
    T: ElementSeq<S, B> + HList,
    T::Storage: HList,
    S: State,
{
    type Storage = HCons<H::Storage, T::Storage>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage {
        HCons {
            head: self.head.render_children(state, backend, context),
            tail: self.tail.render_children(state, backend, context),
        }
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let mut has_changed = false;
        has_changed |= self
            .head
            .update_children(&mut storage.head, state, backend, context);
        has_changed |= self
            .tail
            .update_children(&mut storage.tail, state, backend, context);
        has_changed
    }
}

impl<H, T, S, B> ViewNodeSeq<S, B> for HCons<H, T>
where
    H: ViewNodeSeq<S, B>,
    T: ViewNodeSeq<S, B> + HList,
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
        backend: &B,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut has_changed = false;
        has_changed |= self.head.commit(mode, state, backend, context);
        has_changed |= self.tail.commit(mode, state, backend, context);
        has_changed
    }
}

impl<H, T, Visitor, Context, Output, S, B> Traversable<Visitor, Context, Output, S, B>
    for HCons<H, T>
where
    H: Traversable<Visitor, Context, Output, S, B>,
    T: Traversable<Visitor, Context, Output, S, B> + HList,
    Output: Monoid,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> Output {
        self.head
            .for_each(visitor, state, backend, context)
            .combine(self.tail.for_each(visitor, state, backend, context))
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> Option<Output> {
        self.head
            .search(id_path, visitor, state, backend, context)
            .or_else(|| self.tail.search(id_path, visitor, state, backend, context))
    }
}
