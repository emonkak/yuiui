use std::fmt;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::Store;
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

impl<C, S, M, B> Element<S, M, B> for ComponentElement<C>
where
    C: Component<S, M, B>,
{
    type View = <C::Element as Element<S, M, B>>::View;

    type Components = (
        ComponentNode<C, S, M, B>,
        <C::Element as Element<S, M, B>>::Components,
    );

    fn render(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let component_node = ComponentNode::new(self.component);
        let element = component_node.component.render(store);
        let node = element.render(context, store);
        ViewNode {
            id: node.id,
            state: node.state,
            children: node.children,
            components: (component_node, node.components),
            event_mask: node.event_mask,
            dirty: true,
        }
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let (head_node, tail_nodes) = node.components;
        let element = self.component.render(store);
        head_node.pending_component = Some(self.component);
        *node.dirty = true;
        let mut node = ViewNodeMut {
            id: node.id,
            state: node.state,
            children: node.children,
            components: tail_nodes,
            dirty: node.dirty,
        };
        element.update(&mut node, context, store)
    }
}

impl<C, S, M, B> ElementSeq<S, M, B> for ComponentElement<C>
where
    C: Component<S, M, B>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, B>>::View, <Self as Element<S, M, B>>::Components, S, M, B>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        self.render(context, store)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, store)
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
