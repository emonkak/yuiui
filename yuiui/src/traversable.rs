use crate::id::IdPath;
use crate::state::Store;

pub trait Traversable<Visitor, Context, Output, S, B> {
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> Output;

    fn for_id_path(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> Option<Output>;
}

pub trait Visitor<Node, S, B> {
    type Context;

    type Output: Monoid;

    fn visit(
        &mut self,
        node: &mut Node,
        context: &mut Self::Context,
        store: &mut Store<S>,
        backend: &mut B,
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
