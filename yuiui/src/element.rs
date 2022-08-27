use std::fmt;
use std::marker::PhantomData;

use crate::adapt::Adapt;
use crate::component::Component;
use crate::component_node::{ComponentEnd, ComponentNode, ComponentStack};
use crate::id::IdContext;
use crate::sequence::{ElementSeq, WidgetNodeSeq};
use crate::state::State;
use crate::view::View;
use crate::widget::Widget;
use crate::widget_node::{WidgetNode, WidgetNodeScope, WidgetState};

pub trait Element<S: State, E> {
    type View: View<S, E>;

    type Components: ComponentStack<S, E, View = Self::View>;

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) -> WidgetNode<Self::View, Self::Components, S, E>;

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) -> bool;

    fn adapt<F, OriginState>(self, f: F) -> Adapt<Self, F, S>
    where
        Self: Sized,
        F: Fn(&OriginState) -> &S + Sync + Send + 'static,
    {
        Adapt::new(self, f.into())
    }
}

pub trait DebuggableElement<S: State, E>:
    Element<
        S,
        E,
        View = <Self as DebuggableElement<S, E>>::View,
        Components = <Self as DebuggableElement<S, E>>::Components,
    > + fmt::Debug
{
    type View: View<S, E, Widget = Self::Widget> + fmt::Debug;

    type Widget: Widget<S, E, Children = Self::Children> + fmt::Debug;

    type Children: WidgetNodeSeq<S, E> + fmt::Debug;

    type Components: ComponentStack<S, E, View = <Self as DebuggableElement<S, E>>::View>
        + fmt::Debug;
}

impl<El, S, E> DebuggableElement<S, E> for El
where
    El: Element<S, E> + fmt::Debug,
    El::View: fmt::Debug,
    <El::View as View<S, E>>::Widget: fmt::Debug,
    <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children: fmt::Debug,
    El::Components: fmt::Debug,
    S: State,
{
    type View = El::View;

    type Widget = <El::View as View<S, E>>::Widget;

    type Children = <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children;

    type Components = El::Components;
}

pub struct ViewElement<V: View<S, E>, S: State, E> {
    view: V,
    children: V::Children,
}

impl<V, S, E> ViewElement<V, S, E>
where
    V: View<S, E>,
    S: State,
{
    pub fn new(view: V, children: V::Children) -> Self {
        ViewElement { view, children }
    }
}

impl<V, S, E> Element<S, E> for ViewElement<V, S, E>
where
    V: View<S, E>,
    S: State,
{
    type View = V;

    type Components = ComponentEnd<V>;

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) -> WidgetNode<Self::View, Self::Components, S, E> {
        let id = context.next_identity();
        context.begin_widget(id);
        let children = self.children.render(state, env, context);
        context.end_widget();
        WidgetNode::new(id, self.view, children, ComponentEnd::new())
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) -> bool {
        *scope.state = match scope.state.take().unwrap() {
            WidgetState::Uninitialized(_) => WidgetState::Uninitialized(self.view),
            WidgetState::Prepared(widget, _) => WidgetState::Dirty(widget, self.view),
            WidgetState::Dirty(widget, _) => WidgetState::Dirty(widget, self.view),
        }
        .into();
        self.children.update(scope.children, state, env, context);
        true
    }
}

impl<V, S, E> fmt::Debug for ViewElement<V, S, E>
where
    V: View<S, E> + fmt::Debug,
    V::Children: fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ViewElement")
            .field("view", &self.view)
            .field("children", &self.children)
            .finish()
    }
}

pub struct ComponentElement<C: Component<S, E>, S: State, E> {
    component: C,
    state: PhantomData<S>,
    env: PhantomData<E>,
}

impl<C, S, E> ComponentElement<C, S, E>
where
    C: Component<S, E>,
    S: State,
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
{
    type View = <C::Element as Element<S, E>>::View;

    type Components = (
        ComponentNode<C, S, E>,
        <C::Element as Element<S, E>>::Components,
    );

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) -> WidgetNode<Self::View, Self::Components, S, E> {
        let head_node = ComponentNode::new(self.component);
        let element = head_node.component.render(state, env);
        let widget_node = element.render(state, env, context);
        WidgetNode {
            id: widget_node.id,
            state: widget_node.state,
            children: widget_node.children,
            components: (head_node, widget_node.components),
            event_mask: widget_node.event_mask,
        }
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) -> bool {
        let (head_node, tail_nodes) = scope.components;
        if head_node
            .component
            .should_update(&self.component, state, env)
        {
            let element = self.component.render(state, env);
            let scope = WidgetNodeScope {
                id: scope.id,
                state: scope.state,
                children: scope.children,
                components: tail_nodes,
            };
            head_node.pending_component = Some(self.component);
            element.update(scope, state, env, context)
        } else {
            false
        }
    }
}

impl<C, S, E> fmt::Debug for ComponentElement<C, S, E>
where
    C: Component<S, E> + fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentElement")
            .field("component", &self.component)
            .finish()
    }
}
