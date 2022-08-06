use crate::sequence::ElementSeq;
use crate::widget::Widget;

pub trait View: 'static {
    type Widget: Widget;

    type Children: ElementSeq;

    fn build(&self, children: &<Self::Children as ElementSeq>::Nodes) -> Self::Widget;

    fn rebuild(
        &self,
        children: &<Self::Children as ElementSeq>::Nodes,
        widget: &mut Self::Widget,
    ) -> bool {
        *widget = self.build(children);
        true
    }
}
