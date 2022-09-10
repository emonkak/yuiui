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

impl<'a, V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for UpdateSubtreeVisitor<'a>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    type Context = RenderContext;

    type Output = Vec<(IdPathBuf, Depth)>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, B>,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &mut B,
    ) -> Self::Output {
        if let (Some(&depth), true) = (self.cursor.current().value(), store.dirty()) {
            let is_updated = if depth < CS::LEN {
                CS::update(&mut node.borrow_mut(), depth, 0, context, store, backend)
            } else {
                node.dirty = true;
                node.children.for_each(self, context, store, backend);
                true
            };
            store.mark_clean();
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
                if let Some(child_result) =
                    node.children.search(&[id], self, context, store, backend)
                {
                    result.extend(child_result);
                }
            }
            result
        }
    }
}
