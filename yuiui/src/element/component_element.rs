use std::fmt;
use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::State;
use crate::widget_node::{WidgetNode, WidgetNodeScope};

use super::{Element, ElementSeq};

pub struct ComponentElement<C: Component<S, E>, S: State, E> {
    component: C,
    state: PhantomData<S>,
    env: PhantomData<E>,
}

impl<C, S, E> ComponentElement<C, S, E>
where
    C: Component<S, E>,
    S: State,
{
    pub fn new(component: C) -> ComponentElement<C, S, E> {
        Self {
            component,
            state: PhantomData,
            env: PhantomData,
        }
    }
}

impl<C, S, E> Element<S, E> for ComponentElement<C, S, E>
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
    ) -> WidgetNode<Self::View, Self::Components, S, E> {
        let head_node = ComponentNode::new(self.component);
        let element = head_node.component.render(state, env);
        let widget_node = element.render(state, env, context);
        WidgetNode {
            id: widget_node.id,
            state: widget_node.state,
            children: widget_node.children,
            components: (head_node, widget_node.components),
            event_mask: widget_node.event_mask,
            dirty: true,
        }
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let (head_node, tail_nodes) = scope.components;
        if head_node
            .component
            .should_update(&self.component, state, env)
        {
            let element = self.component.render(state, env);
            head_node.pending_component = Some(self.component);
            *scope.dirty = true;
            let scope = WidgetNodeScope {
                id: scope.id,
                state: scope.state,
                children: scope.children,
                components: tail_nodes,
                dirty: scope.dirty,
            };
            element.update(scope, state, env, context)
        } else {
            false
        }
    }
}

impl<C, S, E> ElementSeq<S, E> for ComponentElement<C, S, E>
where
    C: Component<S, E>,
    S: State,
{
    type Store =
        WidgetNode<<Self as Element<S, E>>::View, <Self as Element<S, E>>::Components, S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        Element::render(self, state, env, context)
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        Element::update(self, store.scope(), state, env, context)
    }
}

impl<C, S, E> fmt::Debug for ComponentElement<C, S, E>
where
    C: Component<S, E> + fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentElement")
            .field("component", &self.component)
            .finish()
    }
}
