use crate::element::{ElementSeq, ViewElement};
use crate::event::{EventTarget, Lifecycle};
use crate::id::IdContext;
use crate::store::Store;

pub trait View<S, M, B>: Sized + for<'event> EventTarget<'event> {
    type Children: ElementSeq<S, M, B>;

    type State;

    #[inline]
    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _backend: &mut B,
    ) {
    }

    #[inline]
    fn event(
        &self,
        _event: <Self as EventTarget>::Event,
        _state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _backend: &mut B,
    ) {
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        store: &Store<S>,
        backend: &mut B,
    ) -> Self::State;

    #[inline]
    fn el(self) -> ViewElement<Self, S, M, B>
    where
        Self::Children: Default,
    {
        ViewElement::new(self, Self::Children::default())
    }

    #[inline]
    fn el_with(self, children: Self::Children) -> ViewElement<Self, S, M, B> {
        ViewElement::new(self, children)
    }
}
