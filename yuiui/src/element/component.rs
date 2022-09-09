use std::fmt;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::Store;
use crate::view::View;
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

    const DEPTH: usize = 1 + <Self::View as View<S, M, B>>::Children::DEPTH;

    fn render(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &B,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let component_node = ComponentNode::new(self.component);
        let element = component_node.render(store, backend);
        let node = element.render(context, store, backend);
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
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &B,
    ) -> bool {
        let (head_node, tail_nodes) = node.components;
        let element = self.component.render(&head_node.state, store, backend);
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
        element.update(&mut node, context, store, backend)
    }
}

impl<C, S, M, B> ElementSeq<S, M, B> for ComponentElement<C>
where
    C: Component<S, M, B>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, B>>::View, <Self as Element<S, M, B>>::Components, S, M, B>;

    const DEPTH: usize = C::Element::DEPTH;

    fn render_children(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &B,
    ) -> Self::Storage {
        self.render(context, store, backend)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &B,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, store, backend)
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
