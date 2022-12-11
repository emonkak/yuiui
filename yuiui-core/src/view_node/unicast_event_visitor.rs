use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::context::CommitContext;
use crate::event::Event;
use crate::id::IdPath;
use crate::view::View;

use super::{Traversable, ViewNode, Visitor};

pub struct UnicastEventVisitor<'a> {
    payload: &'a dyn Any,
    id_path: &'a IdPath,
}

impl<'a> UnicastEventVisitor<'a> {
    pub fn new(payload: &'a dyn Any, id_path: &'a IdPath) -> Self {
        Self { payload, id_path }
    }
}

impl<'a, 'context, V, CS, S, M, E>
    Visitor<ViewNode<V, CS, S, M, E>, CommitContext<'context, S, M, E>> for UnicastEventVisitor<'a>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, E>,
        context: &mut CommitContext<'context, S, M, E>,
    ) {
        if let Some((head, tail)) = self.id_path.split_first() {
            self.id_path = tail;
            node.children.for_id(*head, self, context);
        } else {
            let view = &mut node.view;
            let state = node.view_state.as_mut().unwrap();
            let event = V::Event::from_any(self.payload).unwrap_or_else(|| {
                panic!(
                    "Failed to cast the payload of the event to {}",
                    any::type_name::<V::Event>()
                )
            });
            view.event(event, state, &mut node.children, context);
        }
    }
}
