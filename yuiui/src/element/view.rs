use std::fmt;

use crate::component_stack::ComponentEnd;
use crate::context::{IdContext, RenderContext};
use crate::state::Store;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut, ViewNodeState};

use super::{Element, ElementSeq};

pub struct ViewEl<V: View<S, M, B>, S, M, B> {
    view: V,
    children: V::Children,
}

impl<V, S, M, B> ViewEl<V, S, M, B>
where
    V: View<S, M, B>,
{
    pub fn new(view: V, children: V::Children) -> Self {
        ViewEl { view, children }
    }
}

impl<V, S, M, B> Element<S, M, B> for ViewEl<V, S, M, B>
where
    V: View<S, M, B>,
{
    type View = V;

    type Components = ComponentEnd<V>;

    fn render(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let id = context.next_id();
        context.push_id(id);
        let children = self.children.render_children(context, store);
        let node = ViewNode::new(id, self.view, children, ComponentEnd::new());
        context.pop_id();
        node
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        context.push_id(node.id);

        self.children.update_children(node.children, context, store);

        *node.state = Some(match node.state.take().unwrap() {
            ViewNodeState::Uninitialized(_) => ViewNodeState::Uninitialized(self.view),
            ViewNodeState::Prepared(view, state) | ViewNodeState::Pending(view, _, state) => {
                ViewNodeState::Pending(view, self.view, state)
            }
        });
        *node.dirty = true;

        context.pop_id();

        true
    }
}

impl<V, S, M, B> ElementSeq<S, M, B> for ViewEl<V, S, M, B>
where
    V: View<S, M, B>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, B>>::View, <Self as Element<S, M, B>>::Components, S, M, B>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        self.render(context, store)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        self.update(storage.borrow_mut(), context, store)
    }
}

impl<V, S, M, B> fmt::Debug for ViewEl<V, S, M, B>
where
    V: View<S, M, B> + fmt::Debug,
    V::Children: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ViewEl")
            .field("view", &self.view)
            .field("children", &self.children)
            .finish()
    }
}
