use std::marker::PhantomData;

use crate::adapt::Adapt;
use crate::component::{Component, ComponentNode, ComponentStack};
use crate::context::RenderContext;
use crate::env::{Env, WithEnv};
use crate::sequence::ElementSeq;
use crate::state::State;
use crate::view::View;
use crate::widget::{WidgetNode, WidgetNodeScope, WidgetState};

pub trait Element<S: State, E: for<'a> Env<'a>> {
    type View: View<S, E>;

    type Components: ComponentStack<S, E>;

    fn render(
        self,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> WidgetNode<Self::View, Self::Components, S, E>;

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> bool;

    fn adapt<F, PS>(self, f: F) -> Adapt<Self, F, S>
    where
        Self: Sized,
        F: Fn(&PS) -> &S,
    {
        Adapt::new(self, f.into())
    }

    fn with_env<SE>(self, sub_env: SE) -> WithEnv<Self, SE>
    where
        Self: Sized,
        SE: for<'a> Env<'a>,
    {
        WithEnv::new(self, sub_env.into())
    }
}

#[derive(Debug)]
pub struct ViewElement<V: View<S, E>, S: State, E: for<'a> Env<'a>> {
    view: V,
    children: V::Children,
}

impl<V, S, E> ViewElement<V, S, E>
where
    V: View<S, E>,
    S: State,
    E: for<'a> Env<'a>,
{
    pub fn new(view: V, children: V::Children) -> Self {
        ViewElement { view, children }
    }
}

impl<V, S, E> Element<S, E> for ViewElement<V, S, E>
where
    V: View<S, E>,
    S: State,
    E: for<'a> Env<'a>,
{
    type View = V;

    type Components = ();

    fn render(
        self,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> WidgetNode<Self::View, Self::Components, S, E> {
        let id = context.next_identity();
        context.begin_widget(id);
        let children = self.children.render(state, env, context);
        context.end_widget();
        WidgetNode::new(id, self.view, children, ())
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> bool {
        *scope.state = match scope.state.take().unwrap() {
            WidgetState::Uninitialized(_) => WidgetState::Uninitialized(self.view),
            WidgetState::Prepared(widget) => WidgetState::Changed(widget, self.view),
            WidgetState::Changed(widget, _) => WidgetState::Changed(widget, self.view),
        }
        .into();
        self.children.update(scope.children, state, env, context);
        true
    }
}

#[derive(Debug)]
pub struct ComponentElement<C: Component<S, E>, S: State, E: for<'a> Env<'a>> {
    component: C,
    state: PhantomData<S>,
    env: PhantomData<E>,
}

impl<C, S, E> ComponentElement<C, S, E>
where
    C: Component<S, E>,
    S: State,
    E: for<'a> Env<'a>,
{
    pub fn new(component: C) -> ComponentElement<C, S, E> {
        Self {
            component,
            state: PhantomData,
            env: PhantomData,
        }
    }
}

impl<C, S, E> Element<S, E> for ComponentElement<C, S, E>
where
    C: Component<S, E>,
    S: State,
    E: for<'a> Env<'a>,
{
    type View = <C::Element as Element<S, E>>::View;

    type Components = (
        ComponentNode<C, S, E>,
        <C::Element as Element<S, E>>::Components,
    );

    fn render(
        self,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> WidgetNode<Self::View, Self::Components, S, E> {
        let head_component = ComponentNode::new(self.component);
        let element = head_component.component.render(state, env);
        let widget_node = element.render(state, env, context);
        WidgetNode {
            id: widget_node.id,
            state: widget_node.state,
            children: widget_node.children,
            components: (head_component, widget_node.components),
            event_mask: widget_node.event_mask,
        }
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> bool {
        let (head, tail) = scope.components;
        let scope = WidgetNodeScope {
            id: scope.id,
            state: scope.state,
            children: scope.children,
            components: tail,
        };
        if head.component.should_update(&self.component, state, env) {
            let element = self.component.render(state, env);
            head.pending_component = Some(self.component);
            element.update(scope, state, env, context)
        } else {
            false
        }
    }
}
