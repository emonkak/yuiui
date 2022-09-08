use std::fmt;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::State;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{Element, ElementSeq};

pub struct ComponentElement<C> {
    component: C,
}

impl<C> ComponentElement<C> {
    pub const fn new(component: C) -> ComponentElement<C> {
        Self { component }
    }
}

impl<C, S, B> Element<S, B> for ComponentElement<C>
where
    C: Component<S, B>,
    S: State,
{
    type View = <C::Element as Element<S, B>>::View;

    type Components = (
        ComponentNode<C, S, B>,
        <C::Element as Element<S, B>>::Components,
    );

    fn render(
        self,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        let component_node = ComponentNode::new(self.component);
        let element = component_node.render(state, backend);
        let node = element.render(context, state, backend);
        ViewNode {
            id: node.id,
            state: node.state,
            children: node.children,
            components: (component_node, node.components),
            env: node.env,
            event_mask: node.event_mask,
            dirty: true,
        }
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, B>,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let (head_node, tail_nodes) = node.components;
        let element = self.component.render(&head_node.state, state, backend);
        head_node.pending_component = Some(self.component);
        *node.dirty = true;
        let mut node = ViewNodeMut {
            id: node.id,
            state: node.state,
            children: node.children,
            components: tail_nodes,
            env: node.env,
            dirty: node.dirty,
        };
        element.update(&mut node, context, state, backend)
    }
}

impl<C, S, B> ElementSeq<S, B> for ComponentElement<C>
where
    C: Component<S, B>,
    S: State,
{
    type Storage =
        ViewNode<<Self as Element<S, B>>::View, <Self as Element<S, B>>::Components, S, B>;

    fn render_children(self, context: &mut RenderContext, state: &S, backend: &B) -> Self::Storage {
        self.render(context, state, backend)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, state, backend)
    }
}

impl<C> fmt::Debug for ComponentElement<C>
where
    C: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentElement")
            .field("component", &self.component)
            .finish()
    }
}
