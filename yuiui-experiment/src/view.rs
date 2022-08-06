use crate::element_seq::ElementSeq;
use crate::widget::Widget;

pub trait View: 'static {
    type Widget: Widget;

    type Children: ElementSeq<UINodes = <Self::Widget as Widget>::Children>;

    fn build(&self, children: &<Self::Children as ElementSeq>::VNodes) -> Self::Widget;

    fn rebuild(
        &self,
        children: &<Self::Children as ElementSeq>::VNodes,
        widget: &mut Self::Widget,
    ) -> bool {
        *widget = self.build(children);
        true
    }
}
