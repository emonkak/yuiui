use std::mem;

use crate::component_stack::ComponentStack;
use crate::context::RenderContext;
use crate::id::{Depth, IdPathBuf};
use crate::state::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::ViewNode;

pub struct UpdateVisitor {
    depth: Depth,
}

impl<'a> UpdateVisitor {
    pub fn new(depth: Depth) -> Self {
        Self { depth }
    }
}

impl<V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for UpdateVisitor
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
        backend: &B,
    ) -> Self::Output {
        let depth = mem::replace(&mut self.depth, 0);
        if depth < CS::LEN {
            if CS::update(&mut node.borrow_mut(), depth, 0, context, store, backend) {
                vec![(context.id_path().to_vec(), depth)]
            } else {
                Vec::new()
            }
        } else {
            node.dirty = true;
            node.children.for_each(self, context, store, backend);
            vec![(context.id_path().to_vec(), 0)]
        }
    }
}
