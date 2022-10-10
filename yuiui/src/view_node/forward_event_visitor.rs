use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::id::{IdContext, IdPath};
use crate::store::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::ViewNode;

pub struct ForwardEventVisitor<'a> {
    event: &'a dyn Any,
    id_path: &'a IdPath,
}

impl<'a> ForwardEventVisitor<'a> {
    pub fn new(event: &'a dyn Any, id_path: &'a IdPath) -> Self {
        Self { event, id_path }
    }
}

impl<'a, V, CS, S, M, R> Visitor<ViewNode<V, CS, S, M, R>, S, M, R> for ForwardEventVisitor<'a>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    type Output = Vec<M>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::Output {
        if let Some((head, tail)) = self.id_path.split_first() {
            self.id_path = tail;
            node.children
                .for_id(*head, self, id_context, store, renderer)
                .unwrap_or_default()
        } else {
            let view = &mut node.view;
            let state = node.state.as_mut().unwrap();
            let event: &V::Event = self.event.downcast_ref().unwrap_or_else(|| {
                panic!("Unable to cast event to {}", any::type_name::<V::Event>())
            });
            let mut messages = Vec::new();
            view.event(
                event,
                state,
                &mut node.children,
                id_context,
                store,
                &mut messages,
                renderer,
            );
            messages
        }
    }
}
