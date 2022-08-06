use crate::sequence::ElementSeq;
use crate::widget::Widget;

pub trait View: 'static + Sized {
    type Widget: Widget;

    type Children: ElementSeq<Nodes = <Self::Widget as Widget>::Children>;

    fn build(self, children: &Self::Children) -> Self::Widget;

    fn rebuild(
        self,
        children: &Self::Children,
        widget: &mut Self::Widget,
    ) -> bool {
        *widget = self.build(children);
        true
    }
}
