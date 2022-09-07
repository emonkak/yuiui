use hlist::HNil;

use crate::element::{ElementSeq, ViewElement};
use crate::event::{EventResult, HasEvent, Lifecycle};
use crate::id::IdPath;
use crate::state::State;

pub trait View<S: State, B>: Sized + for<'event> HasEvent<'event> {
    type Widget;

    type Children: ElementSeq<S, B>;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _widget: &mut Self::Widget,
        _children: &<Self::Children as ElementSeq<S, B>>::Storage,
        _id_path: &IdPath,
        _state: &S,
        _backend: &B,
    ) -> EventResult<S> {
        EventResult::nop()
    }

    fn event(
        &self,
        _event: <Self as HasEvent>::Event,
        _widget: &mut Self::Widget,
        _children: &<Self::Children as ElementSeq<S, B>>::Storage,
        _id_path: &IdPath,
        _state: &S,
        _backend: &B,
    ) -> EventResult<S> {
        EventResult::nop()
    }

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, B>>::Storage,
        state: &S,
        backend: &B,
    ) -> Self::Widget;

    fn el(self) -> ViewElement<Self, S, B>
    where
        Self: View<S, B, Children = HNil>,
    {
        ViewElement::new(self, HNil)
    }

    fn el_with(self, children: Self::Children) -> ViewElement<Self, S, B> {
        ViewElement::new(self, children)
    }
}
