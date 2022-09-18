use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::MessageContext;
use crate::event::{Event, HasEvent};
use crate::id::IdPath;
use crate::state::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::{ViewNode, ViewNodeState};

pub struct UpwardEventVisitor<'a> {
    event: &'a dyn Any,
    id_path: &'a IdPath,
}

impl<'a> UpwardEventVisitor<'a> {
    pub fn new(event: &'a dyn Any, id_path: &'a IdPath) -> Self {
        Self { event, id_path }
    }
}

impl<'a, V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for UpwardEventVisitor<'a>
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
        store: &mut Store<S>,
        backend: &mut B,
    ) -> Self::Output {
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, view_state)
            | ViewNodeState::Pending(view, _, view_state) => {
                let mut result = false;
                if let Some((head, tail)) = self.id_path.split_first() {
                    self.id_path = tail;
                    result |= node
                        .children
                        .for_id_path(&[*head], self, context, store, backend)
                        .unwrap_or(false);
                }
                if let Some(event) = <V as HasEvent>::Event::from_any(self.event) {
                    view.event(
                        event,
                        view_state,
                        &node.children,
                        context,
                        store.state(),
                        backend,
                    );
                    result = true;
                }
                result
            }
            _ => false,
        }
    }
}
