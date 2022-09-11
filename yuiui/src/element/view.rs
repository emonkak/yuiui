use std::fmt;

use crate::component_stack::ComponentEnd;
use crate::context::RenderContext;
use crate::state::Store;
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

    fn render(
        self,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        context.with_id(|id, context| {
            let children = self.children.render_children(context, store);
            ViewNode::new(id, self.view, children, ComponentEnd::new())
        })
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> bool {
        context.begin_id(node.id);

        self.children.update_children(node.children, context, store);

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

    fn render_children(self, context: &mut RenderContext, store: &mut Store<S>) -> Self::Storage {
        self.render(context, store)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, store)
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
