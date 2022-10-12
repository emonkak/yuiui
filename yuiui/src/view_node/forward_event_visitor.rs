use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::event::Event;
use crate::id::{IdContext, IdPath};
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

impl<'a, 'context, V, CS, S, M, R>
    Visitor<ViewNode<V, CS, S, M, R>, CommitContext<'context, S, M, R>, S, M, R>
    for ForwardEventVisitor<'a>
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
        if let Some((head, tail)) = self.id_path.split_first() {
            self.id_path = tail;
            node.children.for_id(*head, self, context, id_context);
        } else {
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
                context.renderer,
            );
        }
    }
}
