use std::marker::PhantomData;
use std::mem;

use crate::adapt::Adapt;
use crate::component::{Component, ComponentNode, ComponentStack};
use crate::context::Context;
use crate::sequence::ElementSeq;
use crate::view::View;
use crate::widget::{WidgetNode, WidgetNodeScope, WidgetStatus};

pub trait Element<S> {
    type View: View<S>;

    type Components: ComponentStack<S>;

    fn build(self, state: &S, context: &mut Context)
        -> WidgetNode<Self::View, Self::Components, S>;

    fn rebuild(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S>,
        state: &S,
        context: &mut Context,
    ) -> bool;

    fn adapt<F, NS>(self, f: F) -> Adapt<Self, F, S>
    where
        Self: Sized,
        F: Fn(&NS) -> &S,
    {
        Adapt::new(self, f)
    }
}

#[derive(Debug)]
pub struct ViewElement<V: View<S>, S> {
    view: V,
    children: V::Children,
}

impl<V: View<S>, S> ViewElement<V, S> {
    pub fn new(view: V, children: V::Children) -> Self {
        ViewElement { view, children }
    }
}

impl<V: View<S>, S> Element<S> for ViewElement<V, S> {
    type View = V;

    type Components = ();

    fn build(
        self,
        state: &S,
        context: &mut Context,
    ) -> WidgetNode<Self::View, Self::Components, S> {
        let id = context.next_identity();
        context.push(id);
        let children = self.children.build(state, context);
        let status = WidgetStatus::Uninitialized(self.view);
        context.pop();
        WidgetNode {
            id,
            status: Some(status),
            children,
            components: (),
        }
    }

    fn rebuild(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S>,
        state: &S,
        context: &mut Context,
    ) -> bool {
        *scope.status = match scope.status.take().unwrap() {
            WidgetStatus::Prepared(widget) => WidgetStatus::Changed(widget, self.view),
            WidgetStatus::Changed(widget, _) => WidgetStatus::Changed(widget, self.view),
            WidgetStatus::Uninitialized(_) => WidgetStatus::Uninitialized(self.view),
        }
        .into();
        self.children.rebuild(scope.children, state, context);
        true
    }
}

#[derive(Debug)]
pub struct ComponentElement<C: Component<S>, S> {
    component: C,
    state: PhantomData<S>,
}

impl<C: Component<S>, S> ComponentElement<C, S> {
    pub fn new(component: C) -> ComponentElement<C, S> {
        Self {
            component,
            state: PhantomData,
        }
    }
}

impl<C: Component<S>, S> Element<S> for ComponentElement<C, S> {
    type View = <C::Element as Element<S>>::View;

    type Components = (ComponentNode<C, S>, <C::Element as Element<S>>::Components);

    fn build(
        self,
        state: &S,
        context: &mut Context,
    ) -> WidgetNode<Self::View, Self::Components, S> {
        let component_node = ComponentNode::new(self.component);
        let element = component_node.component.render(state);
        let widget_node = Element::build(element, state, context);
        WidgetNode {
            id: widget_node.id,
            status: widget_node.status,
            children: widget_node.children,
            components: (component_node, widget_node.components),
        }
    }

    fn rebuild(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S>,
        state: &S,
        context: &mut Context,
    ) -> bool {
        let (head, tail) = scope.components;
        let scope = WidgetNodeScope {
            id: scope.id,
            status: scope.status,
            children: scope.children,
            components: tail,
        };
        let old_component = mem::replace(&mut head.component, self.component);
        if old_component.should_update(&head.component, state) {
            let element = head.component.render(state);
            Element::rebuild(element, scope, state, context)
        } else {
            false
        }
    }
}
