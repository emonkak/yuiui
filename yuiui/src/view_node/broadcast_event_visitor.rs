use std::any::{self, Any};

use crate::component_stack::ComponentStack;
use crate::id::{id_tree, IdContext};
use crate::store::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use super::ViewNode;

pub struct BroadcastEventVisitor<'a, R> {
    event: &'a dyn Any,
    cursor: id_tree::Cursor<'a>,
    renderer: &'a mut R,
}

impl<'a, R> BroadcastEventVisitor<'a, R> {
    pub fn new(event: &'a dyn Any, cursor: id_tree::Cursor<'a>, renderer: &'a mut R) -> Self {
        Self {
            event,
            cursor,
            renderer,
        }
    }
}

impl<'a, V, CS, S, M, R> Visitor<ViewNode<V, CS, S, M, R>, S, M, R> for BroadcastEventVisitor<'a, R>
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
        if self.cursor.current().data().is_some() {
            let view = &mut node.view;
            let state = node.state.as_mut().unwrap();
            let event: &V::Event = self.event.downcast_ref().unwrap_or_else(|| {
                panic!("Unable to cast event to {}", any::type_name::<V::Event>())
            });
            view.event(
                &event,
                state,
                &mut node.children,
                id_context,
                store,
                accumulator,
                self.renderer,
            );
        }
        for cursor in self.cursor.children() {
            let id = cursor.current().id();
            self.cursor = cursor;
            node.children
                .for_id(id, self, accumulator, id_context, store);
        }
    }
}
