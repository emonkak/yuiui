use hlist::HNil;

use crate::element::{ElementSeq, ViewElement};
use crate::event::{EventResult, HasEvent, Lifecycle};
use crate::id::IdPath;
use crate::state::State;

pub trait View<S: State, E>: Sized + for<'event> HasEvent<'event> {
    type Widget;

    type Children: ElementSeq<S, E>;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _widget: &mut Self::Widget,
        _children: &<Self::Children as ElementSeq<S, E>>::Storage,
        _id_path: &IdPath,
        _state: &S,
        _env: &E,
    ) -> EventResult<S> {
        EventResult::nop()
    }

    fn event(
        &self,
        _event: <Self as HasEvent>::Event,
        _widget: &mut Self::Widget,
        _children: &<Self::Children as ElementSeq<S, E>>::Storage,
        _id_path: &IdPath,
        _state: &S,
        _env: &E,
    ) -> EventResult<S> {
        EventResult::nop()
    }

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, E>>::Storage,
        state: &S,
        env: &E,
    ) -> Self::Widget;

    fn el(self) -> ViewElement<Self, S, E>
    where
        Self: View<S, E, Children = HNil>,
    {
        ViewElement::new(self, HNil)
    }

    fn el_with(self, children: Self::Children) -> ViewElement<Self, S, E> {
        ViewElement::new(self, children)
    }
}
