use crate::context::MessageContext;
use crate::element::{ElementSeq, ViewEl};
use crate::event::{EventTarget, Lifecycle};
use crate::store::Store;

pub trait View<S, M, R>: Sized + for<'event> EventTarget<'event> {
    type Children: ElementSeq<S, M, R>;

    type State;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut R,
    ) {
    }

    fn event(
        &self,
        _event: <Self as EventTarget>::Event,
        _view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
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
