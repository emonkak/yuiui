use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::MessageContext;
use crate::event::{Event, EventListener};
use crate::state::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::{ViewNode, ViewNodeState};

pub struct DownwardEventVisitor<'a> {
    event: &'a dyn Any,
}

impl<'a> DownwardEventVisitor<'a> {
    pub fn new(event: &'a dyn Any) -> Self {
        Self { event }
    }
}

impl<'a, V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for DownwardEventVisitor<'a>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    type Context = MessageContext<M>;

    type Output = bool;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, B>,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> Self::Output {
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, view_state)
            | ViewNodeState::Pending(view, _, view_state) => {
                let mut result = false;
                if let Some(event) = <V as EventListener>::Event::from_any(self.event) {
                    view.event(
                        event,
                        view_state,
                        &node.children,
                        context,
                        store,
                        backend,
                    );
                    result = true;
                }
                if node.event_mask.contains(&self.event.type_id()) {
                    result |= node.children.for_each(self, context, store, backend);
                }
                result
            }
            _ => false,
        }
    }
}
