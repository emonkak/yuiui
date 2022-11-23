use crate::component_stack::ComponentStack;
use crate::id::{id_tree, Depth, IdContext, IdPathBuf};
use crate::view::View;

use super::{RenderContext, Traversable, ViewNode, Visitor};

pub struct UpdateSubtreeVisitor<'a> {
    cursor: id_tree::Cursor<'a, Depth>,
    result: Vec<(IdPathBuf, Depth)>,
}

impl<'a> UpdateSubtreeVisitor<'a> {
    pub fn new(cursor: id_tree::Cursor<'a, Depth>) -> Self {
        Self {
            cursor,
            result: Vec::new(),
        }
    }

    pub fn into_result(self) -> Vec<(IdPathBuf, Depth)> {
        self.result
    }
}

impl<'a, 'context, V, CS, S, M, E>
    Visitor<ViewNode<V, CS, S, M, E>, RenderContext<'context, S>, S, M, E>
    for UpdateSubtreeVisitor<'a>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, E>,
        context: &mut RenderContext<'context, S>,
        id_context: &mut IdContext,
    ) {
        if let Some(&depth) = self.cursor.current().data() {
            let is_updated = if depth >= CS::DEPTH {
                CS::update(&mut node.into(), depth, context.state, id_context)
            } else {
                node.dirty = true;
                node.children.for_each(self, context, id_context);
                true
            };
            if is_updated {
                self.result.push((id_context.id_path().to_vec(), depth));
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
