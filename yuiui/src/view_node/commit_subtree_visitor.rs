use crate::component_stack::ComponentStack;
use crate::id::{id_tree, Depth, IdContext};
use crate::store::Store;
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

impl<'a, V, CS, S, M, R> Visitor<ViewNode<V, CS, S, M, R>, S, M, R> for CommitSubtreeVisitor<'a>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    type Output = Vec<M>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::Output {
        if let Some(depth) = self.cursor.current().value() {
            let mut messages = Vec::new();
            node.commit_within(
                self.mode,
                *depth,
                id_context,
                store,
                &mut messages,
                renderer,
            );
            messages
        } else {
            let mut result = Vec::new();
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                if let Some(messages) = node.children.for_id(id, self, id_context, store, renderer)
                {
                    result.extend(messages);
                }
            }
            result
        }
    }
}
