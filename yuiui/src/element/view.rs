use std::fmt;

use crate::component_stack::ComponentTermination;
use crate::id::IdStack;
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
        id_stack: &mut IdStack,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        let id = id_stack.next_id();
        id_stack.push_id(id);
        let children = self.children.render_children(id_stack, state);
        let node = ViewNode::new(id, self.view, children, ComponentTermination::new());
        id_stack.pop_id();
        node
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, E>,
        id_stack: &mut IdStack,
        state: &S,
    ) -> bool {
        id_stack.push_id(node.id);
        self.children
            .update_children(node.children, id_stack, state);
        *node.pending_view = Some(self.view);
        *node.dirty = true;
        id_stack.pop_id();
        true
    }
}

impl<V, S, M, E> ElementSeq<S, M, E> for ViewElement<V, S, M, E>
where
    V: View<S, M, E>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, E>>::View, <Self as Element<S, M, E>>::Components, S, M, E>;

    fn render_children(self, id_stack: &mut IdStack, state: &S) -> Self::Storage {
        self.render(id_stack, state)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_stack: &mut IdStack,
        state: &S,
    ) -> bool {
        self.update(storage.into(), id_stack, state)
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
