use std::fmt;

use crate::component_stack::ComponentEnd;
use crate::context::{IdContext, RenderContext};
use crate::state::State;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeScope, ViewNodeState};

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
    ) -> ViewNode<Self::View, Self::Components, S, E> {
        let id = context.next_identity();
        context.begin_view(id);
        let children = self.children.render_children(state, env, context);
        let node = ViewNode::new(id, self.view, children, ComponentEnd::new());
        context.end_view();
        node
    }

    fn update(
        self,
        scope: ViewNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        *scope.state = match scope.state.take().unwrap() {
            ViewNodeState::Uninitialized(_) => ViewNodeState::Uninitialized(self.view),
            ViewNodeState::Prepared(view, widget) | ViewNodeState::Pending(view, _, widget) => {
                ViewNodeState::Pending(view, self.view, widget)
            }
        }
        .into();
        *scope.dirty = true;
        self.children
            .update_children(scope.children, state, env, context);
        true
    }
}

impl<V, S, E> ElementSeq<S, E> for ViewElement<V, S, E>
where
    V: View<S, E>,
    S: State,
{
    type Storage =
        ViewNode<<Self as Element<S, E>>::View, <Self as Element<S, E>>::Components, S, E>;

    fn render_children(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage {
        self.render(state, env, context)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        self.update(storage.scope(), state, env, context)
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
