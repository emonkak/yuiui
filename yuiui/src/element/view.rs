use std::fmt;

use crate::component_stack::ComponentEnd;
use crate::context::RenderContext;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut, ViewNodeState};

use super::{Element, ElementSeq};

pub struct ViewElement<V: View<S, M, B>, S, M, B> {
    view: V,
    children: V::Children,
}

impl<V, S, M, B> ViewElement<V, S, M, B>
where
    V: View<S, M, B>,
{
    pub fn new(view: V, children: V::Children) -> Self {
        ViewElement { view, children }
    }
}

impl<V, S, M, B> Element<S, M, B> for ViewElement<V, S, M, B>
where
    V: View<S, M, B>,
{
    type View = V;

    type Components = ComponentEnd<V>;

    const DEPTH: usize = 1 + V::Children::DEPTH;

    fn render(
        self,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        context.with_id(|id, context| {
            let children = self.children.render_children(context, state, backend);
            ViewNode::new(id, self.view, children, ComponentEnd::new())
        })
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
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

impl<V, S, M, B> ElementSeq<S, M, B> for ViewElement<V, S, M, B>
where
    V: View<S, M, B>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, B>>::View, <Self as Element<S, M, B>>::Components, S, M, B>;

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

impl<V, S, M, B> fmt::Debug for ViewElement<V, S, M, B>
where
    V: View<S, M, B> + fmt::Debug,
    V::Children: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ViewElement")
            .field("view", &self.view)
            .field("children", &self.children)
            .finish()
    }
}
