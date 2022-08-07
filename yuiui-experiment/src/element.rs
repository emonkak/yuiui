use std::mem;

use crate::component::{Component, ComponentNode, ComponentStack};
use crate::context::Context;
use crate::hlist::{HCons, HNil};
use crate::sequence::ElementSeq;
use crate::view::View;
use crate::widget::{WidgetNode, WidgetNodeScope};

pub trait Element: 'static {
    type View: View;

    type Components: ComponentStack;

    fn build(self, context: &mut Context) -> WidgetNode<Self::View, Self::Components>;

    fn rebuild(
        self,
        node: WidgetNodeScope<Self::View, Self::Components>,
        context: &mut Context,
    ) -> bool;
}

#[derive(Debug)]
pub struct ViewElement<V: View> {
    view: V,
    children: V::Children,
}

impl<V: View> ViewElement<V> {
    pub fn new(view: V, children: V::Children) -> Self {
        ViewElement { view, children }
    }
}

impl<V: View> Element for ViewElement<V> {
    type View = V;

    type Components = HNil;

    fn build(self, context: &mut Context) -> WidgetNode<Self::View, Self::Components> {
        let id = context.next_identity();
        context.push(id);
        let children = self.children.build(context);
        let widget = self.view.build(&children);
        context.pop();
        WidgetNode {
            id,
            widget,
            pending_view: None,
            children,
            components: HNil,
        }
    }

    fn rebuild(
        self,
        node: WidgetNodeScope<Self::View, Self::Components>,
        context: &mut Context,
    ) -> bool {
        *node.pending_view = Some(self.view);
        self.children.rebuild(node.children, context);
        true
    }
}

#[derive(Debug)]
pub struct ComponentElement<C: Component> {
    component: C,
}

impl<C: Component> ComponentElement<C> {
    pub fn new(component: C) -> ComponentElement<C> {
        Self { component }
    }
}

impl<C: Component> Element for ComponentElement<C> {
    type View = <C::Element as Element>::View;

    type Components = HCons<ComponentNode<C>, <C::Element as Element>::Components>;

    fn build(self, context: &mut Context) -> WidgetNode<Self::View, Self::Components> {
        let component_node = ComponentNode::new(self.component);
        let widget_node = Element::build(component_node.render(), context);
        WidgetNode {
            id: widget_node.id,
            widget: widget_node.widget,
            pending_view: None,
            children: widget_node.children,
            components: HCons(component_node, widget_node.components),
        }
    }

    fn rebuild(
        self,
        node: WidgetNodeScope<Self::View, Self::Components>,
        context: &mut Context,
    ) -> bool {
        let HCons(head, tail) = node.components;
        let node = WidgetNodeScope {
            id: node.id,
            pending_view: node.pending_view,
            children: node.children,
            components: tail,
        };
        let old_component = mem::replace(&mut head.component, self.component);
        if old_component.should_update(&head.component) {
            Element::rebuild(head.render(), node, context)
        } else {
            false
        }
    }
}
