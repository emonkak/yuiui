use std::mem;

use crate::component::Component;
use crate::context::Context;
use crate::hlist::{HCons, HList, HNil};
use crate::sequence::ElementSeq;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeScope};

pub trait Element: 'static {
    type View: View;

    type Components: HList;

    fn build(self, context: &mut Context) -> ViewNode<Self::View, Self::Components>;

    fn rebuild(
        self,
        node: ViewNodeScope<Self::View, Self::Components>,
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

    type Components = HNil;

    fn build(self, context: &mut Context) -> ViewNode<Self::View, Self::Components> {
        let id = context.next_identity();
        context.push(id);
        let children = self.children.build(context);
        context.pop();
        ViewNode {
            id,
            widget: self.view.build(&children),
            view: self.view,
            children,
            components: HNil,
        }
    }

    fn rebuild(
        self,
        node: ViewNodeScope<Self::View, Self::Components>,
        context: &mut Context,
    ) -> bool {
        *node.children = self.children.build(context);
        *node.widget = self.view.build(node.children);
        *node.view = self.view;
        true
    }
}

#[derive(Debug)]
pub struct ComponentElement<C: Component> {
    component: C,
}

impl<C: Component> Element for ComponentElement<C> {
    type View = <C::Element as Element>::View;

    type Components = HCons<C, <C::Element as Element>::Components>;

    fn build(self, context: &mut Context) -> ViewNode<Self::View, Self::Components> {
        let node = Component::render(&self.component).build(context);
        ViewNode {
            id: node.id,
            widget: node.widget,
            view: node.view,
            children: node.children,
            components: HCons(self.component, node.components),
        }
    }

    fn rebuild(
        self,
        node: ViewNodeScope<Self::View, Self::Components>,
        context: &mut Context,
    ) -> bool {
        let (head_component, node) = node.destruct_components();
        let old_component = mem::replace(head_component, self.component);
        if old_component.should_update(head_component) {
            Component::render(head_component).rebuild(node, context)
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
