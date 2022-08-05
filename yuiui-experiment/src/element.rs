use crate::component::Component;
use crate::context::Context;
use crate::element_seq::ElementSeq;
use crate::node::{UINode, VNode};
use crate::view::View;
use crate::widget::Widget;

pub trait Element: 'static {
    type View: View;

    type Components;

    fn depth() -> usize;

    fn render(
        v_node: &VNode<Self::View, Self::Components>,
    ) -> UINode<<Self::View as View>::Widget> {
        UINode {
            id: v_node.id,
            widget: v_node.view.build(&v_node.children),
            children: <Self::View as View>::Children::render(&v_node.children),
        }
    }

    fn rerender(
        v_node: &VNode<Self::View, Self::Components>,
        widget: &mut <Self::View as View>::Widget,
        children: &mut <<Self::View as View>::Widget as Widget>::Children,
    ) -> bool {
        let mut has_changed = v_node.view.rebuild(&v_node.children, widget);
        has_changed |= <Self::View as View>::Children::rerender(&v_node.children, children);
        has_changed
    }

    fn invalidate(v_node: &VNode<Self::View, Self::Components>, context: &mut Context) {
        context.invalidate(v_node.id);
        <Self::View as View>::Children::invalidate(&v_node.children, context);
    }

    fn build(self, context: &mut Context) -> VNode<Self::View, Self::Components>;

    fn rebuild(
        self,
        view: &mut Self::View,
        children: &mut <<Self::View as View>::Children as ElementSeq>::VNodes,
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

    fn depth() -> usize {
        Self::View::depth()
    }

    fn build(self, context: &mut Context) -> VNode<Self::View, Self::Components> {
        let id = context.next_identity();
        context.push(id);
        let children = self.children.build(context);
        context.pop();
        VNode {
            id,
            view: self.view,
            children,
            components: (),
        }
    }

    fn rebuild(
        self,
        view: &mut Self::View,
        children: &mut <<Self::View as View>::Children as ElementSeq>::VNodes,
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

    fn depth() -> usize {
        Self::View::depth()
    }

    fn build(self, context: &mut Context) -> VNode<Self::View, Self::Components> {
        let v_node = Component::render(&self.component).build(context);
        VNode {
            id: v_node.id,
            view: v_node.view,
            children: v_node.children,
            components: (self.component, v_node.components),
        }
    }

    fn rebuild(
        self,
        view: &mut Self::View,
        children: &mut <<Self::View as View>::Children as ElementSeq>::VNodes,
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
