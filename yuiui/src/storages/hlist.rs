use hlist::{HCons, HList, HNil};
use std::sync::Once;

use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::EventMask;
use crate::id::Id;
use crate::state::Store;
use crate::traversable::{Monoid, Traversable};
use crate::view_node::{CommitMode, ViewNodeSeq};

impl<S, M, R> ElementSeq<S, M, R> for HNil {
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

impl<H, T, S, M, R> ElementSeq<S, M, R> for HCons<H, T>
where
    H: ElementSeq<S, M, R>,
    T: ElementSeq<S, M, R> + HList,
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

impl<S, M, R> ViewNodeSeq<S, M, R> for HNil {
    const SIZE_HINT: (usize, Option<usize>) = (0, Some(0));

    fn event_mask() -> &'static EventMask {
        static MASK: EventMask = EventMask::new();
        &MASK
    }

    fn len(&self) -> usize {
        0
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        None
    }

    fn commit(
        &mut self,
        _mode: CommitMode,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> bool {
        false
    }
}

impl<H, T, S, M, R> ViewNodeSeq<S, M, R> for HCons<H, T>
where
    H: ViewNodeSeq<S, M, R>,
    T: ViewNodeSeq<S, M, R> + HList,
{
    const SIZE_HINT: (usize, Option<usize>) = {
        let (head_lower, head_upper) = H::SIZE_HINT;
        let (tail_lower, tail_upper) = T::SIZE_HINT;
        let lower = head_lower.saturating_add(tail_lower);
        let upper = match (head_upper, tail_upper) {
            (Some(x), Some(y)) => x.checked_add(y),
            _ => None,
        };
        (lower, upper)
    };

    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let head_mask = H::event_mask();
            let tail_mask = T::event_mask();

            INIT.call_once(|| unsafe {
                EVENT_MASK.extend(head_mask);
                EVENT_MASK.extend(tail_mask);
            });
        }

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        self.head.len() + self.tail.len()
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        let head = self.head.id_range();
        let tail = self.tail.id_range();
        match (head, tail) {
            (Some((start, _)), Some((_, end))) => Some((start, end)),
            _ => None,
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        let head_result = self.head.commit(mode, context, store, renderer);
        let tail_result = self.tail.commit(mode, context, store, renderer);
        head_result || tail_result
    }
}

impl<Visitor, Context, Output, S, M, R> Traversable<Visitor, Context, Output, S, M, R> for HNil
where
    Output: Default,
{
    fn for_each(
        &mut self,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Output {
        Output::default()
    }

    fn for_id(
        &mut self,
        _id: Id,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Option<Output> {
        None
    }
}

impl<H, T, Visitor, Context, Output, S, M, R> Traversable<Visitor, Context, Output, S, M, R>
    for HCons<H, T>
where
    H: Traversable<Visitor, Context, Output, S, M, R>,
    T: Traversable<Visitor, Context, Output, S, M, R> + HList,
    Output: Monoid,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Output {
        self.head
            .for_each(visitor, context, store, renderer)
            .combine(self.tail.for_each(visitor, context, store, renderer))
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Option<Output> {
        self.head
            .for_id(id, visitor, context, store, renderer)
            .or_else(|| self.tail.for_id(id, visitor, context, store, renderer))
    }
}
