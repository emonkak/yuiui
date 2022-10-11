use crate::component_stack::ComponentStack;
use crate::id::{id_tree, Depth, IdContext, IdPathBuf};
use crate::view::View;

use super::{RenderContext, Traversable, ViewNode, Visitor};

pub struct UpdateSubtreeVisitor<'a> {
    cursor: id_tree::Cursor<'a, Depth>,
    updated_addresses: Vec<(IdPathBuf, Depth)>,
}

impl<'a> UpdateSubtreeVisitor<'a> {
    pub fn new(cursor: id_tree::Cursor<'a, Depth>) -> Self {
        Self {
            cursor,
            updated_addresses: Vec::new(),
        }
    }

    pub fn into_result(self) -> Vec<(IdPathBuf, Depth)> {
        self.updated_addresses
    }
}

impl<'a, 'context, V, CS, S, M, R>
    Visitor<ViewNode<V, CS, S, M, R>, RenderContext<'context, S>, S, M, R>
    for UpdateSubtreeVisitor<'a>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        context: &mut RenderContext<'context, S>,
        id_context: &mut IdContext,
    ) {
        if let (Some(&depth), true) = (self.cursor.current().data(), context.store.dirty()) {
            context.store.mark_clean();
            let is_updated = if depth < CS::LEN {
                CS::update(node.into(), depth, 0, id_context, context.store)
            } else {
                node.dirty = true;
                node.children.for_each(self, context, id_context);
                true
            };
            if is_updated {
                self.updated_addresses
                    .push((id_context.id_path().to_vec(), depth));
            }
        } else {
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                node.children.for_id(id, self, context, id_context);
            }
        }
    }
}
