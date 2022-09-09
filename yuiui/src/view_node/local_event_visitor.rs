use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::MessageContext;
use crate::event::{Event, HasEvent};
use crate::state::Store;
use crate::traversable::Visitor;
use crate::view::View;

use super::{ViewNode, ViewNodeState};

pub struct LocalEventVisitor<'a> {
    event: &'a dyn Any,
}

impl<'a> LocalEventVisitor<'a> {
    pub fn new(event: &'a dyn Any) -> Self {
        Self { event }
    }
}

impl<'a, V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for LocalEventVisitor<'a>
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
        backend: &B,
    ) -> Self::Output {
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, view_state)
            | ViewNodeState::Pending(view, _, view_state) => {
                let event = <V as HasEvent>::Event::from_any(self.event)
                    .expect("cast any event to view event");
                context.set_depth(CS::LEN);
                view.event(event, view_state, &node.children, context, store, backend);
                true
            }
            ViewNodeState::Uninitialized(_) => false,
        }
    }
}
