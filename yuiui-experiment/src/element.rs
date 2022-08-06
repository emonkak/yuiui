use crate::component::Component;
use crate::context::Context;
use crate::element_seq::ElementSeq;
use crate::node::{UINode, UIStatus};
use crate::view::View;

pub trait Element: 'static {
    type View: View;

    type Components;

    fn build(self, context: &mut Context) -> UINode<Self::View, Self::Components>;

    fn rebuild(
        self,
        view: &mut Self::View,
        children: &mut <<Self::View as View>::Children as ElementSeq>::Nodes,
        components: &mut Self::Components,
        context: &mut Context,
    ) -> bool;
}

#[derive(Debug)]
pub struct ViewElement<V: View> {
    view: V,
    children: V::Children,
}

impl<V: View> Element for ViewElement<V> {
    type View = V;

    type Components = ();

    fn build(self, context: &mut Context) -> UINode<Self::View, Self::Components> {
        let id = context.next_identity();
        context.push(id);
        let children = self.children.build(context);
        context.pop();
        UINode {
            id,
            widget: self.view.build(&children),
            view: self.view,
            children,
            components: (),
            status: UIStatus::Committed,
        }
    }

    fn rebuild(
        self,
        view: &mut Self::View,
        children: &mut <<Self::View as View>::Children as ElementSeq>::Nodes,
        _components: &mut Self::Components,
        context: &mut Context,
    ) -> bool {
        *view = self.view;
        *children = self.children.build(context);
        true
    }
}

#[derive(Debug)]
pub struct ComponentElement<C: Component> {
    component: C,
}

impl<C: Component> Element for ComponentElement<C> {
    type View = <C::Element as Element>::View;

    type Components = (C, <C::Element as Element>::Components);

    fn build(self, context: &mut Context) -> UINode<Self::View, Self::Components> {
        let node = Component::render(&self.component).build(context);
        UINode {
            id: node.id,
            widget: node.widget,
            view: node.view,
            children: node.children,
            components: (self.component, node.components),
            status: node.status,
        }
    }

    fn rebuild(
        self,
        view: &mut Self::View,
        children: &mut <<Self::View as View>::Children as ElementSeq>::Nodes,
        components: &mut Self::Components,
        context: &mut Context,
    ) -> bool {
        let (ref old_component, rest_components) = components;
        if self.component.should_update(old_component) {
            Component::render(&self.component).rebuild(view, children, rest_components, context)
        } else {
            false
        }
    }
}

pub fn view<V: View>(view: V, children: V::Children) -> ViewElement<V> {
    ViewElement { view, children }
}

pub fn component<C: Component>(component: C) -> ComponentElement<C> {
    ComponentElement { component }
}
