use crate::id::Id;
use crate::store::Store;

pub trait Traversable<Visitor, Context, Output, S, M, R> {
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Output;

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Option<Output>;
}

pub trait Visitor<Node, S, R> {
    type Context;

    type Output: Monoid;

    fn visit(
        &mut self,
        node: &mut Node,
        context: &mut Self::Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::Output;
}

pub trait Monoid: Default {
    fn combine(self, other: Self) -> Self;
}

impl Monoid for () {
    fn combine(self, _other: Self) -> Self {
        ()
    }
}

impl Monoid for bool {
    fn combine(self, other: Self) -> Self {
        self || other
    }
}

impl<T> Monoid for Vec<T> {
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}
