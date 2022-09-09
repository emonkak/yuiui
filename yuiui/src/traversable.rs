use crate::id::IdPath;
use crate::state::Store;

pub trait Traversable<Visitor, Context, Output, S, B> {
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        backend: &B,
    ) -> Output;

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        backend: &B,
    ) -> Option<Output>;
}

pub trait Visitor<Node, S, B> {
    type Context;

    type Output: Monoid;

    fn visit(
        &mut self,
        node: &mut Node,
        context: &mut Self::Context,
        store: &Store<S>,
        backend: &B,
    ) -> Self::Output;
}

pub trait Monoid: Default {
    fn combine(self, other: Self) -> Self;
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
