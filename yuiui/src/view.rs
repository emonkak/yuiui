use hlist::HNil;

use crate::element::ViewElement;
use crate::sequence::ElementSeq;
use crate::state::State;
use crate::widget::Widget;

pub trait View<S: State, E>: Sized {
    type Widget: Widget<S, E>;

    type Children: ElementSeq<S, E, Store = <Self::Widget as Widget<S, E>>::Children>;

    fn build(
        self,
        children: &<Self::Widget as Widget<S, E>>::Children,
        state: &S,
        env: &E,
    ) -> Self::Widget;

    fn rebuild(
        self,
        children: &<Self::Widget as Widget<S, E>>::Children,
        widget: &mut Self::Widget,
        state: &S,
        env: &E,
    ) -> bool {
        *widget = self.build(children, state, env);
        true
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
