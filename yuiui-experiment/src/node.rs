use std::fmt;

use crate::context::{Context, Id};
use crate::element_seq::ElementSeq;
use crate::view::View;

pub struct UINode<V: View, CS> {
    pub id: Id,
    pub widget: V::Widget,
    pub view: V,
    pub children: <V::Children as ElementSeq>::Nodes,
    pub components: CS,
    pub status: UIStatus,
}

impl<V: View, CS> UINode<V, CS> {
    // pub fn build(&self) -> UINode<V::Widget> {
    //     UINode {
    //         id: self.id,
    //         widget: self.view.build(&self.children),
    //         children: V::Children::render(&self.children),
    //     }
    // }
    //
    // pub fn rebuild(
    //     &self,
    //     widget: &mut V::Widget,
    //     children: &mut <V::Widget as Widget>::Children,
    // ) -> bool {
    //     let mut has_changed = self.view.rebuild(&self.children, widget);
    //     has_changed |= V::Children::rerender(&self.children, children);
    //     has_changed
    // }

    pub fn invalidate(&mut self, context: &mut Context) {
        context.invalidate(self.id);
        V::Children::invalidate(&mut self.children, context);
    }
}

impl<V, CS> fmt::Debug for UINode<V, CS>
where
    V: View + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Children as ElementSeq>::Nodes: fmt::Debug,
    CS: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UINode")
            .field("id", &self.id)
            .field("view", &self.view)
            .field("widget", &self.view)
            .field("children", &self.children)
            .field("components", &self.components)
            .field("status", &self.status)
            .finish()
    }
}

#[derive(Debug)]
pub enum UIStatus {
    Committed,
    Invalidated,
}
