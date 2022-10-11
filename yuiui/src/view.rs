use crate::element::{ElementSeq, ViewEl};
use crate::event::Lifecycle;
use crate::id::IdContext;
use crate::store::Store;

pub trait View<S, M, R>: Sized {
    type Children: ElementSeq<S, M, R>;

    type State;

    type Event: 'static;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _renderer: &mut R,
    ) {
    }

    fn event(
        &self,
        _event: &Self::Event,
        _state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _renderer: &mut R,
    ) {
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::State;

    fn el(self) -> ViewEl<Self, S, M, R>
    where
        Self::Children: Default,
    {
        ViewEl::new(self, Self::Children::default())
    }

    fn el_with(self, children: Self::Children) -> ViewEl<Self, S, M, R> {
        ViewEl::new(self, children)
    }
}
