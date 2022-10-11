use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::id::{id_tree, IdContext};
use crate::view::View;

use super::{CommitContext, Traversable, ViewNode, Visitor};

pub struct BroadcastEventVisitor<'a> {
    payload: &'a dyn Any,
    cursor: id_tree::Cursor<'a>,
}

impl<'a> BroadcastEventVisitor<'a> {
    pub fn new(payload: &'a dyn Any, cursor: id_tree::Cursor<'a>) -> Self {
        Self { payload, cursor }
    }
}

impl<'a, 'context, V, CS, S, M, R>
    Visitor<ViewNode<V, CS, S, M, R>, CommitContext<'context, S, M, R>, S, M, R>
    for BroadcastEventVisitor<'a>
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
        if self.cursor.current().data().is_some() {
            let view = &mut node.view;
            let state = node.state.as_mut().unwrap();
            let event: &V::Event = self.payload.downcast_ref().unwrap_or_else(|| {
                panic!(
                    "Failed to cast the payload to {}",
                    any::type_name::<V::Event>()
                )
            });
            view.event(
                &event,
                state,
                &mut node.children,
                id_context,
                context.store,
                context.messages,
                context.renderer,
            );
        }
        for cursor in self.cursor.children() {
            let id = cursor.current().id();
            self.cursor = cursor;
            node.children.for_id(id, self, context, id_context);
        }
    }
}
