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

impl<'a, 'context, V, CS, S, M, R>
    Visitor<ViewNode<V, CS, S, M, R>, CommitContext<'context, S, M, R>, S, M, R>
    for CommitSubtreeVisitor<'a>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        context: &mut CommitContext<'context, S, M, R>,
        id_context: &mut IdContext,
    ) {
        if let Some(depth) = self.cursor.current().data() {
            node.commit_within(
                self.mode,
                *depth,
                id_context,
                context.store,
                context.messages,
                context.renderer,
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
