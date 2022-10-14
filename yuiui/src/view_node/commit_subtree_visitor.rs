use crate::component_stack::ComponentStack;
use crate::id::{id_tree, Depth, IdContext};
use crate::view::View;

use super::{CommitContext, CommitMode, Traversable, ViewNode, Visitor};

pub struct CommitSubtreeVisitor<'a> {
    mode: CommitMode,
    cursor: id_tree::Cursor<'a, Depth>,
}

impl<'a> CommitSubtreeVisitor<'a> {
    pub fn new(mode: CommitMode, cursor: id_tree::Cursor<'a, Depth>) -> Self {
        Self { mode, cursor }
    }
}

impl<'a, 'context, V, CS, S, M, B>
    Visitor<ViewNode<V, CS, S, M, B>, CommitContext<'context, S, M, B>, S, M, B>
    for CommitSubtreeVisitor<'a>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, B>,
        context: &mut CommitContext<'context, S, M, B>,
        id_context: &mut IdContext,
    ) {
        if let Some(depth) = self.cursor.current().data() {
            node.commit_from(
                self.mode,
                *depth,
                id_context,
                context.store,
                context.messages,
                context.backend,
            );
        } else {
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                node.children.for_id(id, self, context, id_context);
            }
        }
    }
}
