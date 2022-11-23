use std::fmt;

use crate::component_stack::ComponentTermination;
use crate::context::RenderContext;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{Element, ElementSeq};

pub struct ViewElement<V: View<S, M, E>, S, M, E> {
    view: V,
    children: V::Children,
}

impl<V, S, M, E> ViewElement<V, S, M, E>
where
    V: View<S, M, E>,
{
    pub fn new(view: V, children: V::Children) -> Self {
        ViewElement { view, children }
    }
}

impl<V, S, M, E> Element<S, M, E> for ViewElement<V, S, M, E>
where
    V: View<S, M, E>,
{
    type View = V;

    type Components = ComponentTermination<V>;

    fn render(
        self,
        context: &mut RenderContext<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        let id = context.id_stack.id();
        let children = self.children.render_children(context);
        let node = ViewNode::new(id, self.view, children, ComponentTermination::new());
        node
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, E>,
        context: &mut RenderContext<S>,
    ) -> bool {
        self.children.update_children(node.children, context);
        *node.pending_view = Some(self.view);
        *node.dirty = true;
        true
    }
}

impl<V, S, M, E> ElementSeq<S, M, E> for ViewElement<V, S, M, E>
where
    V: View<S, M, E>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, E>>::View, <Self as Element<S, M, E>>::Components, S, M, E>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        context.render_element(self)
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        context.update_node(self, storage)
    }
}

impl<V, S, M, E> fmt::Debug for ViewElement<V, S, M, E>
where
    V: View<S, M, E> + fmt::Debug,
    V::Children: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ViewElement")
            .field("view", &self.view)
            .field("children", &self.children)
            .finish()
    }
}
