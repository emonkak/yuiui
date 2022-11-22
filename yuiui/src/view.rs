use crate::element::{ElementSeq, ViewElement};
use crate::event::{EventTarget, Lifecycle};
use crate::id::IdStack;
use crate::store::Store;

pub trait View<S, M, E>: Sized + for<'event> EventTarget<'event> {
    type Children: ElementSeq<S, M, E>;

    type State;

    #[inline]
    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _id_stack: &mut IdStack,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _entry_point: &E,
    ) {
    }

    #[inline]
    fn event(
        &self,
        _event: <Self as EventTarget>::Event,
        _state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _id_stack: &mut IdStack,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _entry_point: &E,
    ) {
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        store: &Store<S>,
        entry_point: &E,
    ) -> Self::State;

    #[inline]
    fn el(self, children: Self::Children) -> ViewElement<Self, S, M, E> {
        ViewElement::new(self, children)
    }
}
