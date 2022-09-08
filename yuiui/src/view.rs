use hlist::HNil;

use crate::context::EffectContext;
use crate::effect::EffectOps;
use crate::element::{ElementSeq, ViewElement};
use crate::event::{HasEvent, Lifecycle};
use crate::state::State;

pub trait View<S: State, B>: Sized + for<'event> HasEvent<'event> {
    type Children: ElementSeq<S, B>;

    type State;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, B>>::Storage,
        _context: &EffectContext,
        _state: &S,
        _backend: &B,
    ) -> EffectOps<S> {
        EffectOps::nop()
    }

    fn event(
        &self,
        _event: <Self as HasEvent>::Event,
        _view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, B>>::Storage,
        _context: &EffectContext,
        _state: &S,
        _backend: &B,
    ) -> EffectOps<S> {
        EffectOps::nop()
    }

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, B>>::Storage,
        state: &S,
        backend: &B,
    ) -> Self::State;

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
