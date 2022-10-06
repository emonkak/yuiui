use crate::component_stack::ComponentStack;
use crate::context::RenderContext;
use crate::id::{id_tree, Depth, IdPathBuf};
use crate::state::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::ViewNode;

pub struct UpdateSubtreeVisitor<'a> {
    cursor: id_tree::Cursor<'a, Depth>,
}

impl<'a> UpdateSubtreeVisitor<'a> {
    pub fn new(cursor: id_tree::Cursor<'a, Depth>) -> Self {
        Self { cursor }
    }
}

impl<'a, V, CS, S, M, R> Visitor<ViewNode<V, CS, S, M, R>, S, R> for UpdateSubtreeVisitor<'a>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    type Context = RenderContext;

    type Output = Vec<(IdPathBuf, Depth)>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        context: &mut RenderContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::Output {
        if let (Some(&depth), true) = (self.cursor.current().value(), store.dirty()) {
            store.mark_clean();
            let is_updated = if depth < CS::LEN {
                CS::update(node.into(), depth, 0, context, store)
            } else {
                node.dirty = true;
                node.children.for_each(self, context, store, renderer);
                true
            };
            if is_updated {
                vec![(context.id_path().to_vec(), depth)]
            } else {
                vec![]
            }
        } else {
            let mut result = Vec::new();
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                if let Some(child_result) = node.children.for_id(id, self, context, store, renderer)
                {
                    result.extend(child_result);
                }
            }
            result
        }
    }
}
