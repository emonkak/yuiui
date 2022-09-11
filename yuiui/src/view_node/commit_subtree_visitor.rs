use crate::component_stack::ComponentStack;
use crate::context::MessageContext;
use crate::id::{id_tree, Depth};
use crate::state::{StateTree, Store};
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::{CommitMode, ViewNode};

pub struct CommitSubtreeVisitor<'a> {
    mode: CommitMode,
    cursor: id_tree::Cursor<'a, Depth>,
    state_tree: &'a mut StateTree,
}

impl<'a> CommitSubtreeVisitor<'a> {
    pub fn new(
        mode: CommitMode,
        cursor: id_tree::Cursor<'a, Depth>,
        state_tree: &'a mut StateTree,
    ) -> Self {
        Self {
            mode,
            cursor,
            state_tree,
        }
    }
}

impl<'a, V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for CommitSubtreeVisitor<'a>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    type Context = MessageContext<M>;

    type Output = bool;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, B>,
        context: &mut MessageContext<M>,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> Self::Output {
        if let Some(depth) = self.cursor.current().value() {
            node.commit_within(
                self.mode,
                *depth,
                &mut self.state_tree,
                context,
                store,
                backend,
            )
        } else {
            let mut result = false;
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                result |= node
                    .children
                    .search(&[id], self, context, store, backend)
                    .unwrap_or(false);
            }
            result
        }
    }
}
