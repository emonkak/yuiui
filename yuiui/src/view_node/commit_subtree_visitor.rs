use crate::component_stack::ComponentStack;
use crate::id::{id_tree, Depth, IdContext};
use crate::store::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::{CommitMode, ViewNode};

pub struct CommitSubtreeVisitor<'a, R> {
    mode: CommitMode,
    cursor: id_tree::Cursor<'a, Depth>,
    renderer: &'a mut R,
}

impl<'a, R> CommitSubtreeVisitor<'a, R> {
    pub fn new(mode: CommitMode, cursor: id_tree::Cursor<'a, Depth>, renderer: &'a mut R) -> Self {
        Self {
            mode,
            cursor,
            renderer,
        }
    }
}

impl<'a, V, CS, S, M, R> Visitor<ViewNode<V, CS, S, M, R>, S, M, R> for CommitSubtreeVisitor<'a, R>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    type Accumulator = Vec<M>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        accumulator: &mut Self::Accumulator,
        id_context: &mut IdContext,
        store: &Store<S>,
    ) {
        if let Some(depth) = self.cursor.current().data() {
            node.commit_within(
                self.mode,
                *depth,
                id_context,
                store,
                accumulator,
                self.renderer,
            );
        } else {
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                node.children
                    .for_id(id, self, accumulator, id_context, store);
            }
        }
    }
}
