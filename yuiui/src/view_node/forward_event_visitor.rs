use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::id::{IdContext, IdPath};
use crate::store::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::ViewNode;

pub struct ForwardEventVisitor<'a, R> {
    payload: &'a dyn Any,
    id_path: &'a IdPath,
    renderer: &'a mut R,
}

impl<'a, R> ForwardEventVisitor<'a, R> {
    pub fn new(payload: &'a dyn Any, id_path: &'a IdPath, renderer: &'a mut R) -> Self {
        Self {
            payload,
            id_path,
            renderer,
        }
    }
}

impl<'a, V, CS, S, M, R> Visitor<ViewNode<V, CS, S, M, R>, S, M, R> for ForwardEventVisitor<'a, R>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    type Accumulator = Vec<M>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        accumulator: &mut Self::Accumulator,
        id_context: &mut IdContext,
        store: &Store<S>,
    ) {
        if let Some((head, tail)) = self.id_path.split_first() {
            self.id_path = tail;
            node.children
                .for_id(*head, self, accumulator, id_context, store);
        } else {
            let view = &mut node.view;
            let state = node.state.as_mut().unwrap();
            let event: &V::Event = self.payload.downcast_ref().unwrap_or_else(|| {
                panic!(
                    "Failed to cast the payload to {}",
                    any::type_name::<V::Event>()
                )
            });
            view.event(
                event,
                state,
                &mut node.children,
                id_context,
                store,
                accumulator,
                self.renderer,
            );
        }
    }
}
