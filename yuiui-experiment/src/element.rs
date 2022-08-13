use std::marker::PhantomData;
use std::mem;

use crate::adapt::Adapt;
use crate::component::{Component, ComponentNode, ComponentStack};
use crate::context::RenderContext;
use crate::sequence::ElementSeq;
use crate::state::State;
use crate::view::View;
use crate::widget::{WidgetNode, WidgetNodeScope, WidgetStatus};

pub trait Element<S: State> {
    type View: View<S>;

    type Components: ComponentStack<S>;

    fn render(
        self,
        state: &S,
        context: &mut RenderContext,
    ) -> WidgetNode<Self::View, Self::Components, S>;

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S>,
        state: &S,
        context: &mut RenderContext,
    ) -> bool;

    fn adapt<F, NS>(self, f: F) -> Adapt<Self, F, S>
    where
        Self: Sized,
        F: Fn(&NS) -> &S,
    {
        Adapt::new(self, f.into())
    }
}

#[derive(Debug)]
pub struct ViewElement<V: View<S>, S: State> {
    view: V,
    children: V::Children,
}

impl<V, S> ViewElement<V, S>
where
    V: View<S>,
    S: State,
{
    pub fn new(view: V, children: V::Children) -> Self {
        ViewElement { view, children }
    }
}

impl<V, S> Element<S> for ViewElement<V, S>
where
    V: View<S>,
    S: State,
{
    type View = V;

    type Components = ();

    fn render(
        self,
        state: &S,
        context: &mut RenderContext,
    ) -> WidgetNode<Self::View, Self::Components, S> {
        let id = context.next_identity();
        context.begin_widget(id);
        let children = self.children.render(state, context);
        context.end_widget();
        WidgetNode::new(id, self.view, children, ())
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S>,
        state: &S,
        context: &mut RenderContext,
    ) -> bool {
        *scope.status = match scope.status.take().unwrap() {
            WidgetStatus::Uninitialized(_) => WidgetStatus::Uninitialized(self.view),
            WidgetStatus::Prepared(widget) => WidgetStatus::Changed(widget, self.view),
            WidgetStatus::Changed(widget, _) => WidgetStatus::Changed(widget, self.view),
        }
        .into();
        self.children.update(scope.children, state, context);
        true
    }
}

#[derive(Debug)]
pub struct ComponentElement<C: Component<S>, S: State> {
    component: C,
    state: PhantomData<S>,
}

impl<C: Component<S>, S: State> ComponentElement<C, S> {
    pub fn new(component: C) -> ComponentElement<C, S> {
        Self {
            component,
            state: PhantomData,
        }
    }
}

impl<C, S> Element<S> for ComponentElement<C, S>
where
    C: Component<S>,
    S: State,
{
    type View = <C::Element as Element<S>>::View;

    type Components = (ComponentNode<C, S>, <C::Element as Element<S>>::Components);

    fn render(
        self,
        state: &S,
        context: &mut RenderContext,
    ) -> WidgetNode<Self::View, Self::Components, S> {
        let head_component = ComponentNode::new(self.component);
        let element = head_component.component.render(state);
        element
            .render(state, context)
            .map_components(|components| (head_component, components))
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S>,
        state: &S,
        context: &mut RenderContext,
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
            Element::update(element, scope, state, context)
        } else {
            false
        }
    }
}
