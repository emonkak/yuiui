use crate::context::CommitContext;
use crate::element::{ElementSeq, ViewElement};
use crate::event::{EventTarget, Lifecycle};

pub trait View<S, M, E>: Sized + for<'event> EventTarget<'event> {
    type Children: ElementSeq<S, M, E>;

    type State;

    #[inline]
    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _context: &mut CommitContext<S, M, E>,
    ) {
    }

    #[inline]
    fn event(
        &self,
        _event: <Self as EventTarget>::Event,
        _view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _context: &mut CommitContext<S, M, E>,
    ) {
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) -> Self::State;

    #[inline]
    fn el(self, children: Self::Children) -> ViewElement<Self, S, M, E> {
        ViewElement::new(self, children)
    }
}
