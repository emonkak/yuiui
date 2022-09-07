use crate::component_stack::ComponentStack;
use crate::context::IdContext;
use crate::element::ElementSeq;
use crate::id::{ComponentIndex, Cursor, Id, IdPathBuf};
use crate::state::State;
use crate::traversable::Traversable;
use crate::traversable::TraversableVisitor;
use crate::view::View;
use crate::view_node::ViewNode;

pub struct BatchVisitor<'a, Visitor> {
    cursor: Cursor<'a, ComponentIndex>,
    visitor_factory: fn(Id, ComponentIndex) -> Visitor,
    changed_nodes: Vec<(IdPathBuf, ComponentIndex)>,
}

impl<'a, Visitor> BatchVisitor<'a, Visitor> {
    pub fn new(
        cursor: Cursor<'a, ComponentIndex>,
        visitor_factory: fn(Id, ComponentIndex) -> Visitor,
    ) -> Self {
        Self {
            cursor,
            visitor_factory,
            changed_nodes: Vec::new(),
        }
    }

    pub fn into_changed_nodes(self) -> Vec<(IdPathBuf, ComponentIndex)> {
        self.changed_nodes
    }
}

impl<'a, Visitor, Context, V, CS, S, B> TraversableVisitor<ViewNode<V, CS, S, B>, Context, S, B>
    for BatchVisitor<'a, Visitor>
where
    Visitor: TraversableVisitor<ViewNode<V, CS, S, B>, Context, S, B>,
    Context: IdContext,
    V: View<S, B>,
    <<V as View<S, B>>::Children as ElementSeq<S, B>>::Storage:
        Traversable<BatchVisitor<'a, Visitor>, Context, S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, B>,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> bool {
        let current = self.cursor.current();
        if let Some(component_index) = current.value() {
            let mut visitor = (self.visitor_factory)(current.id(), *component_index);
            if visitor.visit(node, state, backend, context) {
                self.changed_nodes
                    .push((context.id_path().to_vec(), *component_index));
                true
            } else {
                false
            }
        } else {
            let mut result = false;
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                result |= node.children.search(&[id], self, state, backend, context);
            }
            result
        }
    }
}
