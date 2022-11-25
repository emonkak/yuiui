use crate::component::Component;
use crate::component_stack::ComponentStack;
use crate::context::RenderContext;
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

    type Components = (C, <C::Element as Element<S, M, E>>::Components);

    fn render(
        self,
        context: &mut RenderContext<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        context.level = Self::Components::LEVEL;
        let element = self.component.render(context);
        let node = element.render(context);
        ViewNode {
            id: node.id,
            view: node.view,
            pending_view: node.pending_view,
            view_state: node.view_state,
            children: node.children,
            components: (self.component, node.components),
            dirty: true,
        }
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, E>,
        context: &mut RenderContext<S>,
    ) -> bool {
        let (head_component, tail_components) = node.components;
        context.level = Self::Components::LEVEL;
        let element = self.component.render(context);
        *head_component = self.component;
        let mut node = ViewNodeMut {
            id: node.id,
            view: node.view,
            pending_view: node.pending_view,
            view_state: node.view_state,
            children: node.children,
            components: tail_components,
            dirty: node.dirty,
        };
        element.update(&mut node, context)
    }
}

impl<C, S, M, E> ElementSeq<S, M, E> for ComponentElement<C>
where
    C: Component<S, M, E>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, E>>::View, <Self as Element<S, M, E>>::Components, S, M, E>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        context.render_node(self)
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        context.update_node(self, storage)
    }
}
