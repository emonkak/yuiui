use hlist::{HCons, HList, HNil};
use std::ops::RangeInclusive;
use std::sync::Once;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::Id;
use crate::state::Store;
use crate::traversable::{Monoid, Traversable};
use crate::view_node::{CommitMode, ViewNodeRange, ViewNodeSeq};

impl<S, M, B> ElementSeq<S, M, B> for HNil {
    type Storage = HNil;

    fn render_children(self, _context: &mut RenderContext, _store: &Store<S>) -> Self::Storage {
        HNil
    }

    fn update_children(
        self,
        _nodes: &mut Self::Storage,
        _context: &mut RenderContext,
        _store: &Store<S>,
    ) -> bool {
        false
    }
}

impl<H, T, S, M, B> ElementSeq<S, M, B> for HCons<H, T>
where
    H: ElementSeq<S, M, B>,
    T: ElementSeq<S, M, B> + HList,
    T::Storage: HList,
{
    type Storage = HCons<H::Storage, T::Storage>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        HCons {
            head: self.head.render_children(context, store),
            tail: self.tail.render_children(context, store),
        }
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let mut has_changed = false;
        has_changed |= self.head.update_children(&mut storage.head, context, store);
        has_changed |= self.tail.update_children(&mut storage.tail, context, store);
        has_changed
    }
}

impl<S, M, B> ViewNodeSeq<S, M, B> for HNil {
    const IS_DYNAMIC: bool = false;

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
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> bool {
        false
    }
}

impl<H, T, S, M, B> ViewNodeSeq<S, M, B> for HCons<H, T>
where
    H: ViewNodeSeq<S, M, B>,
    T: ViewNodeSeq<S, M, B> + HList,
{
    const IS_DYNAMIC: bool = H::IS_DYNAMIC && T::IS_DYNAMIC;

    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let head_mask = H::event_mask();
            let tail_mask = T::event_mask();

            INIT.call_once(|| unsafe {
                if !head_mask.is_empty() {
                    EVENT_MASK.extend(head_mask);
                }
                if !tail_mask.is_empty() {
                    EVENT_MASK.extend(tail_mask);
                }
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
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let head_result = self.head.commit(mode, context, store, backend);
        let tail_result = self.tail.commit(mode, context, store, backend);
        head_result || tail_result
    }
}

impl<H, T> ViewNodeRange for HCons<H, T>
where
    H: ViewNodeRange,
    T: ViewNodeRange + HList,
{
    fn id_range(&self) -> RangeInclusive<Id> {
        let head = self.head.id_range();
        let tail = self.tail.id_range();
        *head.start()..=*tail.end()
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
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Output {
        Output::default()
    }

    fn for_id(
        &mut self,
        _id: Id,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Option<Output> {
        None
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
        store: &Store<S>,
        backend: &mut B,
    ) -> Output {
        self.head
            .for_each(visitor, context, store, backend)
            .combine(self.tail.for_each(visitor, context, store, backend))
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        backend: &mut B,
    ) -> Option<Output> {
        self.head
            .for_id(id, visitor, context, store, backend)
            .or_else(|| self.tail.for_id(id, visitor, context, store, backend))
    }
}
