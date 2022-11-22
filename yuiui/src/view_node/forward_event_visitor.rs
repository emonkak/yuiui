use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::event::Event;
use crate::id::{IdPath, IdStack};
use crate::view::View;

use super::{CommitContext, Traversable, ViewNode, Visitor};

pub struct ForwardEventVisitor<'a> {
    payload: &'a dyn Any,
    id_path: &'a IdPath,
}

impl<'a> ForwardEventVisitor<'a> {
    pub fn new(payload: &'a dyn Any, id_path: &'a IdPath) -> Self {
        Self { payload, id_path }
    }
}

impl<'a, 'context, V, CS, S, M, E>
    Visitor<ViewNode<V, CS, S, M, E>, CommitContext<'context, S, M, E>, S, M, E>
    for ForwardEventVisitor<'a>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, E>,
        context: &mut CommitContext<'context, S, M, E>,
        id_stack: &mut IdStack,
    ) {
        if let Some((head, tail)) = self.id_path.split_first() {
            self.id_path = tail;
            node.children.for_id(*head, self, context, id_stack);
        } else {
            let view = &mut node.view;
            let state = node.view_state.as_mut().unwrap();
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
                id_stack,
                context.store,
                context.messages,
                context.entry_point,
            );
        }
    }
}
