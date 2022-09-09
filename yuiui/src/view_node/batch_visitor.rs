use crate::component_stack::ComponentStack;
use crate::element::ElementSeq;
use crate::id::{Cursor, Depth, Id};
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

impl<'a, Inner, V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for BatchVisitor<'a, Inner>
where
    Inner: Visitor<ViewNode<V, CS, S, M, B>, S, B>,
    V: View<S, M, B>,
    <V::Children as ElementSeq<S, M, B>>::Storage:
        Traversable<Self, Inner::Context, Inner::Output, S, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    type Context = Inner::Context;

    type Output = Inner::Output;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, B>,
        context: &mut Self::Context,
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
