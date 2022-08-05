use std::fmt;

use crate::element_seq::ElementSeq;
use crate::widget::Widget;
use crate::context::Id;

pub trait View: 'static {
    type Widget: Widget;

    type Children: ElementSeq<Widgets = <Self::Widget as Widget>::Children>;

    fn depth() -> usize {
        1 + Self::Children::depth()
    }

    fn build(&self, children: &<Self::Children as ElementSeq>::Views) -> Self::Widget;

    fn rebuild(
        &self,
        children: &<Self::Children as ElementSeq>::Views,
        widget: &mut Self::Widget,
    ) -> bool {
        *widget = View::build(self, children);
        true
    }
}

pub struct ViewPod<V: View, C> {
    pub(crate) id: Id,
    pub(crate) view: V,
    pub(crate) children: <V::Children as ElementSeq>::Views,
    pub(crate) components: C,
}

impl<V, C> fmt::Debug for ViewPod<V, C>
where
    V: View + fmt::Debug,
    <V::Children as ElementSeq>::Views: fmt::Debug,
    C: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ViewPod")
            .field("id", &self.id)
            .field("view", &self.view)
            .field("children", &self.children)
            .field("components", &self.components)
            .finish()
    }
}
