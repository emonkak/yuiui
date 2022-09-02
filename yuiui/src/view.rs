use hlist::HNil;

use crate::effect::EffectPath;
use crate::element::{ElementSeq, ViewElement};
use crate::event::{EventResult, HasEvent, Lifecycle};
use crate::state::State;

pub trait View<S: State, E>: Sized + for<'event> HasEvent<'event> {
    type Widget;

    type Children: ElementSeq<S, E>;

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, E>>::Store,
        state: &S,
        env: &E,
    ) -> Self::Widget;

    fn rebuild(
        &self,
        children: &<Self::Children as ElementSeq<S, E>>::Store,
        widget: &mut Self::Widget,
        state: &S,
        env: &E,
    ) -> bool {
        *widget = self.build(children, state, env);
        true
    }

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _widget: &mut Self::Widget,
        _children: &<Self::Children as ElementSeq<S, E>>::Store,
        _effect_path: &EffectPath,
        _state: &S,
        _env: &E,
    ) -> EventResult<S> {
        EventResult::nop()
    }

    fn event(
        &self,
        _event: <Self as HasEvent>::Event,
        _widget: &mut Self::Widget,
        _children: &<Self::Children as ElementSeq<S, E>>::Store,
        _effect_path: &EffectPath,
        _state: &S,
        _env: &E,
    ) -> EventResult<S> {
        EventResult::nop()
    }

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
