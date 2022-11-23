use crate::element::{ElementSeq, ViewElement};
use crate::event::{EventTarget, Lifecycle};
use crate::id::IdContext;

pub trait View<S, M, E>: Sized + for<'event> EventTarget<'event> {
    type Children: ElementSeq<S, M, E>;

    type State;

    #[inline]
    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _state: &S,
        _messages: &mut Vec<M>,
        _entry_point: &E,
        _id_context: &mut IdContext,
    ) {
    }

    #[inline]
    fn event(
        &self,
        _event: <Self as EventTarget>::Event,
        _view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _state: &S,
        _messages: &mut Vec<M>,
        _entry_point: &E,
        _id_context: &mut IdContext,
    ) {
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        state: &S,
        entry_point: &E,
    ) -> Self::State;

    #[inline]
    fn el(self, children: Self::Children) -> ViewElement<Self, S, M, E> {
        ViewElement::new(self, children)
    }
}
