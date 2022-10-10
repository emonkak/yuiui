use crate::id::{Id, IdContext};
use crate::store::Store;

pub trait Traversable<Visitor, Output, S, M, R> {
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Output
    where
        Output: Monoid;

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Option<Output>;
}

pub trait Visitor<Node, S, M, R> {
    type Output;

    fn visit(
        &mut self,
        node: &mut Node,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::Output;
}

pub trait Monoid: Default {
    fn combine(self, other: Self) -> Self;
}

impl Monoid for () {
    #[inline]
    fn combine(self, _other: Self) -> Self {
        ()
    }
}

impl Monoid for bool {
    #[inline]
    fn combine(self, other: Self) -> Self {
        self || other
    }
}

impl<T> Monoid for Vec<T> {
    #[inline]
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}
