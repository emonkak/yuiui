use std::fmt;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::Store;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{Element, ElementSeq};

pub struct ComponentEl<C> {
    component: C,
}

impl<C> ComponentEl<C> {
    pub const fn new(component: C) -> ComponentEl<C> {
        Self { component }
    }
}

impl<C, S, M, R> Element<S, M, R> for ComponentEl<C>
where
    C: Component<S, M, R>,
{
    type View = <C::Element as Element<S, M, R>>::View;

    type Components = (
        ComponentNode<C, S, M, R>,
        <C::Element as Element<S, M, R>>::Components,
    );

    fn render(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, R> {
        let component_node = ComponentNode::new(self.component);
        let element = component_node.component().render(store);
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
        node: ViewNodeMut<Self::View, Self::Components, S, M, R>,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let (head_node, tail_nodes) = node.components;
        let element = self.component.render(store);
        head_node.update(self.component);
        *node.dirty = true;
        let node = ViewNodeMut {
            id: node.id,
            state: node.state,
            children: node.children,
            components: tail_nodes,
            dirty: node.dirty,
        };
        element.update(node, context, store)
    }
}

impl<C, S, M, R> ElementSeq<S, M, R> for ComponentEl<C>
where
    C: Component<S, M, R>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, R>>::View, <Self as Element<S, M, R>>::Components, S, M, R>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        self.render(context, store)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        self.update(storage.borrow_mut(), context, store)
    }
}

impl<C> fmt::Debug for ComponentEl<C>
where
    C: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentEl")
            .field("component", &self.component)
            .finish()
    }
}
