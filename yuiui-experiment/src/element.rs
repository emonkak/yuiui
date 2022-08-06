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

    fn build(
        self,
        context: &mut Context,
    ) -> WidgetNode<<Self::View as View>::Widget, Self::Components>;

    fn rebuild(
        self,
        node: WidgetNodeScope<<Self::View as View>::Widget, Self::Components>,
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

    fn build(
        self,
        context: &mut Context,
    ) -> WidgetNode<<Self::View as View>::Widget, Self::Components> {
        let id = context.next_identity();
        context.push(id);
        let widget = self.view.build(&self.children);
        let children = self.children.build(context);
        context.pop();
        WidgetNode {
            id,
            widget,
            children,
            components: HNil,
        }
    }

    fn rebuild(
        self,
        node: WidgetNodeScope<<Self::View as View>::Widget, Self::Components>,
        context: &mut Context,
    ) -> bool {
        *node.widget = self.view.build(&self.children);
        *node.children = self.children.build(context);
        true
    }
}

#[derive(Debug)]
pub struct ComponentElement<C: Component> {
    component: C,
}

impl<C: Component> Element for ComponentElement<C> {
    type View = <C::Element as Element>::View;

    type Components = HCons<ComponentNode<C>, <C::Element as Element>::Components>;

    fn build(
        self,
        context: &mut Context,
    ) -> WidgetNode<<Self::View as View>::Widget, Self::Components> {
        let component_node = ComponentNode::new(self.component);
        let widget_node = component_node.render().build(context);
        WidgetNode {
            id: widget_node.id,
            widget: widget_node.widget,
            children: widget_node.children,
            components: HCons(component_node, widget_node.components),
        }
    }

    fn rebuild(
        self,
        node: WidgetNodeScope<<Self::View as View>::Widget, Self::Components>,
        context: &mut Context,
    ) -> bool {
        let HCons(head, tail) = node.components;
        let node = WidgetNodeScope {
            id: node.id,
            widget: node.widget,
            children: node.children,
            components: tail,
        };
        let old_component = mem::replace(&mut head.component, self.component);
        if old_component.should_update(&head.component) {
            head.render().rebuild(node, context)
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
