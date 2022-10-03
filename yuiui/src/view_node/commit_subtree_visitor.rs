use crate::component_stack::ComponentStack;
use crate::context::MessageContext;
use crate::id::{id_tree, Depth};
use crate::state::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::{CommitMode, ViewNode};

pub struct CommitSubtreeVisitor<'a> {
    mode: CommitMode,
    cursor: id_tree::Cursor<'a, Depth>,
}

impl<'a> CommitSubtreeVisitor<'a> {
    pub fn new(mode: CommitMode, cursor: id_tree::Cursor<'a, Depth>) -> Self {
        Self { mode, cursor }
    }
}

impl<'a, V, CS, S, M, R> Visitor<ViewNode<V, CS, S, M, R>, S, R> for CommitSubtreeVisitor<'a>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    type Context = MessageContext<M>;

    type Output = bool;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::Output {
        if let Some(depth) = self.cursor.current().value() {
            node.commit_within(self.mode, *depth, context, store, renderer)
        } else {
            let mut result = false;
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                result |= node
                    .children
                    .for_id(id, self, context, store, renderer)
                    .unwrap_or(false);
            }
            result
        }
    }
}
