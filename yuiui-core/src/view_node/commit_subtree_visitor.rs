use crate::component_stack::ComponentStack;
use crate::context::CommitContext;
use crate::id::id_tree;
use crate::view::View;

use super::{CommitMode, Traversable, ViewNode, Visitor};

pub struct CommitSubtreeVisitor<'a> {
    mode: CommitMode,
    cursor: id_tree::Cursor<'a, ()>,
}

impl<'a> CommitSubtreeVisitor<'a> {
    pub fn new(mode: CommitMode, cursor: id_tree::Cursor<'a, ()>) -> Self {
        Self { mode, cursor }
    }
}

impl<'a, 'context, V, CS, S, M, E>
    Visitor<ViewNode<V, CS, S, M, E>, CommitContext<'context, S, M, E>> for CommitSubtreeVisitor<'a>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, E>,
        context: &mut CommitContext<'context, S, M, E>,
    ) {
        if self.cursor.current().data().is_some() {
            node.commit_whole(self.mode, context);
        } else {
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                node.children.for_id(id, self, context);
            }
        }
    }
}
