use crate::element::ViewElement;
use crate::sequence::ElementSeq;
use crate::state::State;
use crate::widget::Widget;

pub trait View<S: State>: Sized {
    type Widget: Widget<S>;

    type Children: ElementSeq<S, Store = <Self::Widget as Widget<S>>::Children>;

    fn build(self, children: &<Self::Widget as Widget<S>>::Children, state: &S) -> Self::Widget;

    fn rebuild(
        self,
        children: &<Self::Widget as Widget<S>>::Children,
        widget: &mut Self::Widget,
        state: &S,
    ) -> bool {
        *widget = self.build(children, state);
        true
    }

    fn el(self) -> ViewElement<Self, S>
    where
        Self::Children: Default,
    {
        ViewElement::new(self, Default::default())
    }

    fn el_with(self, children: Self::Children) -> ViewElement<Self, S> {
        ViewElement::new(self, children)
    }
}
