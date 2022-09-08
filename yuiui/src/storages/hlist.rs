use hlist::{HCons, HList, HNil};
use std::sync::Once;

use crate::context::{EffectContext, RenderContext};
use crate::effect::EffectOps;
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::IdPath;
use crate::traversable::{Monoid, Traversable};
use crate::view_node::{CommitMode, ViewNodeSeq};

impl<S, M, B> ElementSeq<S, M, B> for HNil {
    type Storage = HNil;

    const DEPTH: usize = 0;

    fn render_children(
        self,
        _context: &mut RenderContext,
        _state: &S,
        _backend: &B,
    ) -> Self::Storage {
        HNil
    }

    fn update_children(
        self,
        _nodes: &mut Self::Storage,
        _context: &mut RenderContext,
        _state: &S,
        _backend: &B,
    ) -> bool {
        false
    }
}

impl<S, M, B> ViewNodeSeq<S, M, B> for HNil {
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
        _context: &mut EffectContext,
        _state: &S,
        _backend: &B,
    ) -> EffectOps<M> {
        EffectOps::nop()
    }
}

impl<Visitor, Context, Output, S, B> Traversable<Visitor, Context, Output, S, B> for HNil
where
    Output: Default,
{
    fn for_each(
        &mut self,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _state: &S,
        _backend: &B,
    ) -> Output {
        Output::default()
    }

    fn search(
        &mut self,
        _id_path: &IdPath,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _state: &S,
        _backend: &B,
    ) -> Option<Output> {
        None
    }
}

impl<H, T, S, M, B> ElementSeq<S, M, B> for HCons<H, T>
where
    H: ElementSeq<S, M, B>,
    T: ElementSeq<S, M, B> + HList,
    T::Storage: HList,
{
    type Storage = HCons<H::Storage, T::Storage>;

    const DEPTH: usize = [H::DEPTH, T::DEPTH][(H::DEPTH < T::DEPTH) as usize];

    fn render_children(self, context: &mut RenderContext, state: &S, backend: &B) -> Self::Storage {
        HCons {
            head: self.head.render_children(context, state, backend),
            tail: self.tail.render_children(context, state, backend),
        }
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let mut has_changed = false;
        has_changed |= self
            .head
            .update_children(&mut storage.head, context, state, backend);
        has_changed |= self
            .tail
            .update_children(&mut storage.tail, context, state, backend);
        has_changed
    }
}

impl<H, T, S, M, B> ViewNodeSeq<S, M, B> for HCons<H, T>
where
    H: ViewNodeSeq<S, M, B>,
    T: ViewNodeSeq<S, M, B> + HList,
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
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<M> {
        self.head
            .commit(mode, context, state, backend)
            .combine(self.tail.commit(mode, context, state, backend))
    }
}

impl<H, T, Visitor, Context, Output, S, B> Traversable<Visitor, Context, Output, S, B>
    for HCons<H, T>
where
    H: Traversable<Visitor, Context, Output, S, B>,
    T: Traversable<Visitor, Context, Output, S, B> + HList,
    Output: Monoid,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Output {
        self.head
            .for_each(visitor, context, state, backend)
            .combine(self.tail.for_each(visitor, context, state, backend))
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Option<Output> {
        self.head
            .search(id_path, visitor, context, state, backend)
            .or_else(|| self.tail.search(id_path, visitor, context, state, backend))
    }
}
