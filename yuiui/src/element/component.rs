use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{Element, ElementSeq};

#[derive(Debug)]
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
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, R> {
        let component_node = ComponentNode::new(self.component);
        let element = component_node.component().render(state);
        let node = element.render(context, state);
        ViewNode {
            id: node.id,
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
        node: ViewNodeMut<Self::View, Self::Components, S, M, R>,
        context: &mut RenderContext,
        state: &S,
    ) -> bool {
        let (head_node, tail_nodes) = node.components;
        let element = self.component.render(state);
        head_node.update(self.component);
        *node.dirty = true;
        let node = ViewNodeMut {
            id: node.id,
            view: node.view,
            pending_view: node.pending_view,
            state: node.state,
            children: node.children,
            components: tail_nodes,
            dirty: node.dirty,
        };
        element.update(node, context, state)
    }
}

impl<C, S, M, R> ElementSeq<S, M, R> for ComponentEl<C>
where
    C: Component<S, M, R>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, R>>::View, <Self as Element<S, M, R>>::Components, S, M, R>;

    fn render_children(self, context: &mut RenderContext, state: &S) -> Self::Storage {
        self.render(context, state)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
    ) -> bool {
        self.update(storage.into(), context, state)
    }
}
