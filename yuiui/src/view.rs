use crate::element::{ElementSeq, ViewEl};
use crate::event::{EventTarget, Lifecycle};
use crate::id::IdContext;
use crate::store::Store;

pub trait View<S, M, B>: Sized + for<'event> EventTarget<'event> {
    type Children: ElementSeq<S, M, B>;

    type State;

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

    fn el(self) -> ViewEl<Self, S, M, B>
    where
        Self::Children: Default,
    {
        ViewEl::new(self, Self::Children::default())
    }

    fn el_with(self, children: Self::Children) -> ViewEl<Self, S, M, B> {
        ViewEl::new(self, children)
    }
}
