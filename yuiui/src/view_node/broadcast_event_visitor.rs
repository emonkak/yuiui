use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::event::Event;
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

impl<'a, 'context, V, CS, S, M, E>
    Visitor<ViewNode<V, CS, S, M, E>, CommitContext<'context, S, M, E>, S, M, E>
    for BroadcastEventVisitor<'a>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, E>,
        context: &mut CommitContext<'context, S, M, E>,
        id_context: &mut IdContext,
    ) {
        if self.cursor.current().data().is_some() {
            let view = &mut node.view;
            let state = node.state.as_mut().unwrap();
            let event = V::Event::from_any(self.payload).unwrap_or_else(|| {
                panic!(
                    "Failed to cast the payload of the event to {}",
                    any::type_name::<V::Event>()
                )
            });
            view.event(
                event,
                state,
                &mut node.children,
                id_context,
                context.store,
                context.messages,
                context.entry_point,
            );
        }
        for cursor in self.cursor.children() {
            let id = cursor.current().id();
            self.cursor = cursor;
            node.children.for_id(id, self, context, id_context);
        }
    }
}
