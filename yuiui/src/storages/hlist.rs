use hlist::{HCons, HList, HNil};

use crate::element::ElementSeq;
use crate::id::{Id, IdContext};
use crate::view_node::{CommitMode, Traversable, ViewNodeSeq};

impl<S, M, E> ElementSeq<S, M, E> for HNil {
    type Storage = HNil;

    fn render_children(self, _state: &S, _id_context: &mut IdContext) -> Self::Storage {
        HNil
    }

    fn update_children(
        self,
        _nodes: &mut Self::Storage,
        _state: &S,
        _id_context: &mut IdContext,
    ) -> bool {
        false
    }
}

impl<H, T, S, M, E> ElementSeq<S, M, E> for HCons<H, T>
where
    H: ElementSeq<S, M, E>,
    T: ElementSeq<S, M, E> + HList,
    T::Storage: HList,
{
    type Storage = HCons<H::Storage, T::Storage>;

    fn render_children(self, state: &S, id_context: &mut IdContext) -> Self::Storage {
        HCons {
            head: self.head.render_children(state, id_context),
            tail: self.tail.render_children(state, id_context),
        }
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        id_context: &mut IdContext,
    ) -> bool {
        let mut has_changed = false;
        has_changed |= self
            .head
            .update_children(&mut storage.head, state, id_context);
        has_changed |= self
            .tail
            .update_children(&mut storage.tail, state, id_context);
        has_changed
    }
}

impl<S, M, E> ViewNodeSeq<S, M, E> for HNil {
    const SIZE_HINT: (usize, Option<usize>) = (0, Some(0));

    fn len(&self) -> usize {
        0
    }

    fn commit(
        &mut self,
        _mode: CommitMode,
        _state: &S,
        _id_context: &mut IdContext,
        _messages: &mut Vec<M>,
        _entry_point: &E,
    ) -> bool {
        false
    }

    fn gc(&mut self) {}
}

impl<H, T, S, M, E> ViewNodeSeq<S, M, E> for HCons<H, T>
where
    H: ViewNodeSeq<S, M, E>,
    T: ViewNodeSeq<S, M, E> + HList,
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

    fn len(&self) -> usize {
        self.head.len() + self.tail.len()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        id_context: &mut IdContext,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool {
        let head_result = self
            .head
            .commit(mode, state, id_context, messages, entry_point);
        let tail_result = self
            .tail
            .commit(mode, state, id_context, messages, entry_point);
        head_result || tail_result
    }

    fn gc(&mut self) {
        self.head.gc();
        self.tail.gc();
    }
}

impl<Visitor, Context, S, M, E> Traversable<Visitor, Context, S, M, E> for HNil {
    fn for_each(
        &mut self,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _id_context: &mut IdContext,
    ) {
    }

    fn for_id(
        &mut self,
        _id: Id,
        _visitor: &mut Visitor,
        _context: &mut Context,
        _id_context: &mut IdContext,
    ) -> bool {
        false
    }
}

impl<H, T, Visitor, Context, S, M, E> Traversable<Visitor, Context, S, M, E> for HCons<H, T>
where
    H: Traversable<Visitor, Context, S, M, E>,
    T: Traversable<Visitor, Context, S, M, E> + HList,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    ) {
        self.head.for_each(visitor, context, id_context);
        self.tail.for_each(visitor, context, id_context);
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    ) -> bool {
        if self.head.for_id(id, visitor, context, id_context) {
            return true;
        }
        if self.tail.for_id(id, visitor, context, id_context) {
            return true;
        }
        false
    }
}
