use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::component_stack::ComponentStack;
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
        state: &S,
        id_context: &mut IdContext,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        id_context.set_depth(<Self::Components as ComponentStack<S, M, E>>::DEPTH);
        let component_node = ComponentNode::new(self.component);
        let element = component_node.render(state, id_context);
        let node = element.render(state, id_context);
        ViewNode {
            id: node.id,
            view: node.view,
            pending_view: node.pending_view,
            view_state: node.view_state,
            children: node.children,
            components: (component_node, node.components),
            dirty: true,
        }
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, E>,
        state: &S,
        id_context: &mut IdContext,
    ) -> bool {
        let (head_component, tail_components) = node.components;
        let element = self.component.render(state, id_context);
        head_component.update(self.component);
        let node = ViewNodeMut {
            id: node.id,
            view: node.view,
            pending_view: node.pending_view,
            view_state: node.view_state,
            children: node.children,
            components: tail_components,
            dirty: node.dirty,
        };
        element.update(node, state, id_context)
    }
}

impl<C, S, M, E> ElementSeq<S, M, E> for ComponentElement<C>
where
    C: Component<S, M, E>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, E>>::View, <Self as Element<S, M, E>>::Components, S, M, E>;

    fn render_children(self, state: &S, id_context: &mut IdContext) -> Self::Storage {
        self.render(state, id_context)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        id_context: &mut IdContext,
    ) -> bool {
        self.update(storage.into(), state, id_context)
    }
}
