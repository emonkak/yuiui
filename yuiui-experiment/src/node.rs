use std::fmt;

use crate::context::{Context, Id};
use crate::element_seq::ElementSeq;
use crate::view::View;
use crate::widget::Widget;

pub struct VNode<V: View, C> {
    pub id: Id,
    pub view: V,
    pub children: <V::Children as ElementSeq>::VNodes,
    pub components: C,
}

impl<V: View, C> VNode<V, C> {
    pub fn build(&self) -> UINode<V::Widget> {
        UINode {
            id: self.id,
            widget: self.view.build(&self.children),
            children: V::Children::render(&self.children),
        }
    }

    pub fn rebuild(
        &self,
        widget: &mut V::Widget,
        children: &mut <V::Widget as Widget>::Children,
    ) -> bool {
        let mut has_changed = self.view.rebuild(&self.children, widget);
        has_changed |= V::Children::rerender(&self.children, children);
        has_changed
    }

    pub fn invalidate(&self, context: &mut Context) {
        context.invalidate(self.id);
        V::Children::invalidate(&self.children, context);
    }
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
