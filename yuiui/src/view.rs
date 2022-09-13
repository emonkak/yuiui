use hlist::HNil;

use crate::context::MessageContext;
use crate::element::{ElementSeq, ViewElement};
use crate::event::{HasEvent, Lifecycle};

pub trait View<S, M, B>: Sized + for<'event> HasEvent<'event> {
    type Children: ElementSeq<S, M, B>;

    type State;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _view_state: &mut Self::State,
        _children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        _context: &mut MessageContext<M>,
        _state: &S,
        _backend: &mut B,
    ) {
    }

    fn event(
        &self,
        _event: <Self as HasEvent>::Event,
        _view_state: &mut Self::State,
        _children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        _context: &mut MessageContext<M>,
        _state: &S,
        _backend: &mut B,
    ) {
    }

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        state: &S,
        backend: &mut B,
    ) -> Self::State;

    fn el(self) -> ViewElement<Self, S, M, B>
    where
        Self: View<S, M, B, Children = HNil>,
    {
        ViewElement::new(self, HNil)
    }

    fn el_with(self, children: Self::Children) -> ViewElement<Self, S, M, B> {
        ViewElement::new(self, children)
    }
}
