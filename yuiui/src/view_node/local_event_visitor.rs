use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::context::MessageContext;
use crate::event::{Event, EventListener};
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
        backend: &mut B,
    ) -> Self::Output {
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, view_state)
            | ViewNodeState::Pending(view, _, view_state) => {
                let event =
                    <V as EventListener>::Event::from_any(self.event).unwrap_or_else(|| {
                        panic!(
                            "cast any event to {}",
                            any::type_name::<<V as EventListener>::Event>()
                        )
                    });
                view.event(
                    event,
                    view_state,
                    &mut node.children,
                    context,
                    store,
                    backend,
                );
                true
            }
            ViewNodeState::Uninitialized(_) => false,
        }
    }
}
