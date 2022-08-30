use std::fmt;

use crate::component_node::ComponentEnd;
use crate::context::{IdContext, RenderContext};
use crate::state::State;
use crate::view::View;
use crate::widget_node::{WidgetNode, WidgetNodeScope, WidgetState};

use super::{Element, ElementSeq};

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
        context: &mut RenderContext,
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
        context: &mut RenderContext,
    ) -> bool {
        *scope.state = match scope.state.take().unwrap() {
            WidgetState::Uninitialized(_) => WidgetState::Uninitialized(self.view),
            WidgetState::Prepared(widget, view) => WidgetState::Pending(widget, view, self.view),
            WidgetState::Dirty(widget, view) => WidgetState::Pending(widget, view, self.view),
            WidgetState::Pending(widget, view, _) => WidgetState::Pending(widget, view, self.view),
        }
        .into();
        *scope.dirty = true;
        self.children.update(scope.children, state, env, context);
        true
    }
}

impl<V, S, E> ElementSeq<S, E> for ViewElement<V, S, E>
where
    V: View<S, E>,
    S: State,
{
    type Store =
        WidgetNode<<Self as Element<S, E>>::View, <Self as Element<S, E>>::Components, S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        Element::render(self, state, env, context)
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        Element::update(self, store.scope(), state, env, context)
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
