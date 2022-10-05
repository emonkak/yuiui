use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::MessageContext;
use crate::event::{Event, EventListener};
use crate::id::IdPath;
use crate::state::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::ViewNode;

pub struct DownwardEventVisitor<'a> {
    event: &'a dyn Any,
    id_path: &'a IdPath,
}

impl<'a> DownwardEventVisitor<'a> {
    pub fn new(event: &'a dyn Any, id_path: &'a IdPath) -> Self {
        Self { event, id_path }
    }
}

impl<'a, V, CS, S, M, R> Visitor<ViewNode<V, CS, S, M, R>, S, R> for DownwardEventVisitor<'a>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    type Context = MessageContext<M>;

    type Output = bool;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::Output {
        if let Some((head, tail)) = self.id_path.split_first() {
            self.id_path = tail;
            node.children
                .for_id(*head, self, context, store, renderer)
                .unwrap_or(false)
        } else {
            let mut result = false;
            if node.event_mask.contains(&self.event.type_id()) {
                if let Some(event) = <V as EventListener>::Event::from_any(self.event) {
                    let view = &mut node.view;
                    let state = node.state.as_mut().unwrap();
                    view.event(event, state, &mut node.children, context, store, renderer);
                    result = true;
                }
                result |= node.children.for_each(self, context, store, renderer);
            }
            result
        }
    }
}
