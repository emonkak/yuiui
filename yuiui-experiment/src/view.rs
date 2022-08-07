use crate::element::ViewElement;
use crate::sequence::ElementSeq;
use crate::widget::Widget;

pub trait View: 'static + Sized {
    type Widget: Widget;

    type Children: ElementSeq<Store = <Self::Widget as Widget>::Children>;

    fn build(self, children: &<Self::Widget as Widget>::Children) -> Self::Widget;

    fn rebuild(
        self,
        children: &<Self::Widget as Widget>::Children,
        widget: &mut Self::Widget,
    ) -> bool {
        *widget = self.build(children);
        true
    }

    fn el(self) -> ViewElement<Self>
    where
        Self::Children: Default,
    {
        ViewElement::new(self, Default::default())
    }

    fn el_with(self, children: Self::Children) -> ViewElement<Self> {
        ViewElement::new(self, children)
    }
}
