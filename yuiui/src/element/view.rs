use std::fmt;

use crate::component_stack::ComponentEnd;
use crate::id::IdContext;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{Element, ElementSeq};

pub struct ViewElement<V: View<S, M, B>, S, M, B> {
    view: V,
    children: V::Children,
}

impl<V, S, M, B> ViewElement<V, S, M, B>
where
    V: View<S, M, B>,
{
    pub fn new(view: V, children: V::Children) -> Self {
        ViewElement { view, children }
    }
}

impl<V, S, M, B> Element<S, M, B> for ViewElement<V, S, M, B>
where
    V: View<S, M, B>,
{
    type View = V;

    type Components = ComponentEnd<V>;

    fn render(
        self,
        id_context: &mut IdContext,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let id = id_context.next_id();
        id_context.push_id(id);
        let children = self.children.render_children(id_context, state);
        let node = ViewNode::new(id, self.view, children, ComponentEnd::new());
        id_context.pop_id();
        node
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, B>,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        id_context.push_id(node.id);

        self.children
            .update_children(node.children, id_context, state);

        *node.pending_view = Some(self.view);
        *node.dirty = true;

        id_context.pop_id();

        true
    }
}

impl<V, S, M, B> ElementSeq<S, M, B> for ViewElement<V, S, M, B>
where
    V: View<S, M, B>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, B>>::View, <Self as Element<S, M, B>>::Components, S, M, B>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
        self.render(id_context, state)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        self.update(storage.into(), id_context, state)
    }
}

impl<V, S, M, B> fmt::Debug for ViewElement<V, S, M, B>
where
    V: View<S, M, B> + fmt::Debug,
    V::Children: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ViewElement")
            .field("view", &self.view)
            .field("children", &self.children)
            .finish()
    }
}
