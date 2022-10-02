use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::context::MessageContext;
use crate::event::{Event, EventListener};
use crate::id::IdPath;
use crate::state::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::{ViewNode, ViewNodeState};

pub struct LocalEventVisitor<'a> {
    event: &'a dyn Any,
    id_path: &'a IdPath,
}

impl<'a> LocalEventVisitor<'a> {
    pub fn new(event: &'a dyn Any, id_path: &'a IdPath) -> Self {
        Self { event, id_path }
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
        if let Some((head, tail)) = self.id_path.split_first() {
            self.id_path = tail;
            node.children
                .for_id(*head, self, context, store, backend)
                .unwrap_or(false)
        } else {
            match node.state.as_mut().unwrap() {
                ViewNodeState::Prepared(view, state) | ViewNodeState::Pending(view, _, state) => {
                    let event =
                        <V as EventListener>::Event::from_any(self.event).unwrap_or_else(|| {
                            panic!(
                                "cast the event to {}",
                                any::type_name::<<V as EventListener>::Event>()
                            )
                        });
                    view.event(event, state, &mut node.children, context, store, backend);
                    true
                }
                ViewNodeState::Uninitialized(_) => false,
            }
        }
    }
}
