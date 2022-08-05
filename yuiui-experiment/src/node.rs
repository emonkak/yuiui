use std::fmt;

use crate::context::Id;
use crate::element_seq::ElementSeq;
use crate::view::View;
use crate::widget::Widget;

pub struct VNode<V: View, C> {
    pub id: Id,
    pub view: V,
    pub children: <V::Children as ElementSeq>::VNodes,
    pub components: C,
}

impl<V, C> fmt::Debug for VNode<V, C>
where
    V: View + fmt::Debug,
    <V::Children as ElementSeq>::VNodes: fmt::Debug,
    C: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VNode")
            .field("id", &self.id)
            .field("view", &self.view)
            .field("children", &self.children)
            .field("components", &self.components)
            .finish()
    }
}

#[derive(Debug)]
pub struct UINode<W: Widget> {
    pub id: Id,
    pub widget: W,
    pub children: W::Children,
}
