use std::fmt;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::State;
use crate::view_node::{ViewNode, ViewNodeScope};

use super::{Element, ElementSeq};

pub struct ComponentElement<C> {
    component: C,
}

impl<C> ComponentElement<C> {
    pub const fn new(component: C) -> ComponentElement<C> {
        Self { component }
    }
}

impl<C, S, E> Element<S, E> for ComponentElement<C>
where
    C: Component<S, E>,
    S: State,
{
    type View = <C::Element as Element<S, E>>::View;

    type Components = (
        ComponentNode<C, S, E>,
        <C::Element as Element<S, E>>::Components,
    );

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, E> {
        let component_node = ComponentNode::new(self.component);
        let element = component_node.render(state, env);
        let view_node = element.render(state, env, context);
        ViewNode {
            id: view_node.id,
            state: view_node.state,
            children: view_node.children,
            components: (component_node, view_node.components),
            event_mask: view_node.event_mask,
            dirty: true,
        }
    }

    fn update(
        self,
        scope: ViewNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let (head_node, tail_nodes) = scope.components;
        let element = self.component.render(state, env);
        head_node.pending_component = Some(self.component);
        *scope.dirty = true;
        let scope = ViewNodeScope {
            id: scope.id,
            state: scope.state,
            children: scope.children,
            components: tail_nodes,
            dirty: scope.dirty,
        };
        element.update(scope, state, env, context)
    }
}

impl<C, S, E> ElementSeq<S, E> for ComponentElement<C>
where
    C: Component<S, E>,
    S: State,
{
    type Storage =
        ViewNode<<Self as Element<S, E>>::View, <Self as Element<S, E>>::Components, S, E>;

    fn render_children(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage {
        self.render(state, env, context)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        self.update(storage.scope(), state, env, context)
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
