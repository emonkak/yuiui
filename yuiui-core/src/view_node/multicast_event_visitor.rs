use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::context::CommitContext;
use crate::event::Event;
use crate::id::id_tree;
use crate::view::View;

use super::{Traversable, ViewNode, Visitor};

pub struct MulticastEventVisitor<'a> {
    payload: &'a dyn Any,
    cursor: id_tree::Cursor<'a, ()>,
}

impl<'a> MulticastEventVisitor<'a> {
    pub fn new(payload: &'a dyn Any, cursor: id_tree::Cursor<'a, ()>) -> Self {
        Self { payload, cursor }
    }
}

impl<'a, 'context, V, CS, S, M, E>
    Visitor<ViewNode<V, CS, S, M, E>, CommitContext<'context, S, M, E>>
    for MulticastEventVisitor<'a>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, E>,
        context: &mut CommitContext<'context, S, M, E>,
    ) {
        if self.cursor.current().data().is_some() {
            let view = &mut node.view;
            let view_state = node.view_state.as_mut().unwrap();
            let event = V::Event::from_any(self.payload).unwrap_or_else(|| {
                panic!(
                    "Failed to cast the payload of the event to {}",
                    any::type_name::<V::Event>()
                )
            });
            view.event(event, view_state, &mut node.children, context);
        }
        for cursor in self.cursor.children() {
            let id = cursor.current().id();
            self.cursor = cursor;
            node.children.for_id(id, self, context);
        }
    }
}
