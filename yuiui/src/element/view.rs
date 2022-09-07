use std::fmt;

use crate::component_stack::ComponentEnd;
use crate::context::RenderContext;
use crate::state::State;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeScope, ViewNodeState};

use super::{Element, ElementSeq};

pub struct ViewElement<V: View<S, B>, S: State, B> {
    view: V,
    children: V::Children,
}

impl<V, S, B> ViewElement<V, S, B>
where
    V: View<S, B>,
    S: State,
{
    pub fn new(view: V, children: V::Children) -> Self {
        ViewElement { view, children }
    }
}

impl<V, S, B> Element<S, B> for ViewElement<V, S, B>
where
    V: View<S, B>,
    S: State,
{
    type View = V;

    type Components = ComponentEnd<V>;

    fn render(
        self,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        context.with_identity(|id, context| {
            let children = self.children.render_children(state, backend, context);
            ViewNode::new(id, self.view, children, ComponentEnd::new())
        })
    }

    fn update(
        self,
        scope: &mut ViewNodeScope<Self::View, Self::Components, S, B>,
        state: &S,
        backend: &B,
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
            .update_children(scope.children, state, backend, context);
        true
    }
}

impl<V, S, B> ElementSeq<S, B> for ViewElement<V, S, B>
where
    V: View<S, B>,
    S: State,
{
    type Storage =
        ViewNode<<Self as Element<S, B>>::View, <Self as Element<S, B>>::Components, S, B>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage {
        self.render(state, backend, context)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        self.update(&mut storage.scope(), state, backend, context)
    }
}

impl<V, S, B> fmt::Debug for ViewElement<V, S, B>
where
    V: View<S, B> + fmt::Debug,
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
