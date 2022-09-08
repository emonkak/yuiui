use crate::component_stack::ComponentStack;
use crate::element::ElementSeq;
use crate::id::{Cursor, Depth, Id};
use crate::state::State;
use crate::traversable::{Monoid, Traversable, Visitor};
use crate::view::View;
use crate::view_node::ViewNode;

pub struct BatchVisitor<'a, Visitor> {
    cursor: Cursor<'a, Depth>,
    visitor_factory: fn(Id, Depth) -> Visitor,
}

impl<'a, Visitor> BatchVisitor<'a, Visitor> {
    pub fn new(cursor: Cursor<'a, Depth>, visitor_factory: fn(Id, Depth) -> Visitor) -> Self {
        Self {
            cursor,
            visitor_factory,
        }
    }
}

impl<'a, Inner, Context, V, CS, S, B> Visitor<ViewNode<V, CS, S, B>, Context, S, B>
    for BatchVisitor<'a, Inner>
where
    Inner: Visitor<ViewNode<V, CS, S, B>, Context, S, B>,
    V: View<S, B>,
    <<V as View<S, B>>::Children as ElementSeq<S, B>>::Storage:
        Traversable<Self, Context, Inner::Output, S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    type Output = Inner::Output;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, B>,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Self::Output {
        let current = self.cursor.current();
        if let Some(depth) = current.value() {
            let mut visitor = (self.visitor_factory)(current.id(), *depth);
            visitor.visit(node, context, state, backend)
        } else {
            let mut result = Self::Output::default();
            for cursor in self.cursor.children() {
                let id = cursor.current().id();
                self.cursor = cursor;
                if let Some(child_result) =
                    node.children.search(&[id], self, context, state, backend)
                {
                    result = result.combine(child_result);
                }
            }
            result
        }
    }
}
