use hlist::{HCons, HList, HNil};

use crate::context::{CommitContext, RenderContext};
use crate::element::ElementSeq;
use crate::id::Id;
use crate::view_node::{CommitMode, Traversable, ViewNodeSeq};

impl<S, M, E> ElementSeq<S, M, E> for HNil {
    type Storage = HNil;

    fn render_children(self, _context: &mut RenderContext<S>) -> Self::Storage {
        HNil
    }

    fn update_children(self, _nodes: &mut Self::Storage, _context: &mut RenderContext<S>) -> bool {
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

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        HCons {
            head: self.head.render_children(context),
            tail: self.tail.render_children(context),
        }
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        let mut has_changed = false;
        has_changed |= self.head.update_children(&mut storage.head, context);
        has_changed |= self.tail.update_children(&mut storage.tail, context);
        has_changed
    }
}

impl<S, M, E> ViewNodeSeq<S, M, E> for HNil {
    const SIZE_HINT: (usize, Option<usize>) = (0, Some(0));

    fn len(&self) -> usize {
        0
    }

    fn commit(&mut self, _mode: CommitMode, _context: &mut CommitContext<S, M, E>) -> bool {
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

    fn commit(&mut self, mode: CommitMode, context: &mut CommitContext<S, M, E>) -> bool {
        let head_result = self.head.commit(mode, context);
        let tail_result = self.tail.commit(mode, context);
        head_result || tail_result
    }

    fn gc(&mut self) {
        self.head.gc();
        self.tail.gc();
    }
}

impl<Visitor, Context> Traversable<Visitor, Context> for HNil {
    fn for_each(&mut self, _visitor: &mut Visitor, _context: &mut Context) {}

    fn for_id(&mut self, _id: Id, _visitor: &mut Visitor, _context: &mut Context) -> bool {
        false
    }
}

impl<H, T, Visitor, Context> Traversable<Visitor, Context> for HCons<H, T>
where
    H: Traversable<Visitor, Context>,
    T: Traversable<Visitor, Context> + HList,
{
    fn for_each(&mut self, visitor: &mut Visitor, context: &mut Context) {
        self.head.for_each(visitor, context);
        self.tail.for_each(visitor, context);
    }

    fn for_id(&mut self, id: Id, visitor: &mut Visitor, context: &mut Context) -> bool {
        if self.head.for_id(id, visitor, context) {
            return true;
        }
        if self.tail.for_id(id, visitor, context) {
            return true;
        }
        false
    }
}
