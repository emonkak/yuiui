use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::id::IdContext;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{Element, ElementSeq};

#[derive(Debug)]
pub struct ComponentElement<C> {
    component: C,
}

impl<C> ComponentElement<C> {
    pub const fn new(component: C) -> ComponentElement<C> {
        Self { component }
    }
}

impl<C, S, M, E> Element<S, M, E> for ComponentElement<C>
where
    C: Component<S, M, E>,
{
    type View = <C::Element as Element<S, M, E>>::View;

    type Components = (
        ComponentNode<C, S, M, E>,
        <C::Element as Element<S, M, E>>::Components,
    );

    fn render(
        self,
        id_context: &mut IdContext,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        let element = self.component.render(state);
        let node = element.render(id_context, state);
        let component_node = ComponentNode::new(self.component, node.depth);
        ViewNode {
            id: node.id,
            depth: node.depth + 1,
            view: node.view,
            pending_view: node.pending_view,
            state: node.state,
            children: node.children,
            components: (component_node, node.components),
            dirty: true,
        }
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, E>,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        let (head_component, tail_components) = node.components;
        let element = self.component.render(state);
        head_component.update(self.component);
        *node.dirty = true;
        let node = ViewNodeMut {
            id: node.id,
            depth: node.depth,
            view: node.view,
            pending_view: node.pending_view,
            state: node.state,
            children: node.children,
            components: tail_components,
            dirty: node.dirty,
        };
        element.update(node, id_context, state)
    }
}

impl<C, S, M, E> ElementSeq<S, M, E> for ComponentElement<C>
where
    C: Component<S, M, E>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, E>>::View, <Self as Element<S, M, E>>::Components, S, M, E>;

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
