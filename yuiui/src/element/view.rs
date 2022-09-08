use std::fmt;

use crate::component_stack::ComponentEnd;
use crate::context::RenderContext;
use crate::state::State;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut, ViewNodeState};

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

    const DEPTH: usize = 1 + V::Children::DEPTH;

    fn render(
        self,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        context.with_id(|id, context| {
            let children = self.children.render_children(context, state, backend);
            ViewNode::new(id, self.view, children, ComponentEnd::new())
        })
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, B>,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        context.begin_id(node.id);

        self.children
            .update_children(node.children, context, state, backend);

        *node.state = Some(match node.state.take().unwrap() {
            ViewNodeState::Uninitialized(_) => ViewNodeState::Uninitialized(self.view),
            ViewNodeState::Prepared(view, view_state)
            | ViewNodeState::Pending(view, _, view_state) => {
                ViewNodeState::Pending(view, self.view, view_state)
            }
        });
        *node.dirty = true;

        context.end_id();

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

    const DEPTH: usize = 1 + V::Children::DEPTH;

    fn render_children(self, context: &mut RenderContext, state: &S, backend: &B) -> Self::Storage {
        self.render(context, state, backend)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, state, backend)
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
