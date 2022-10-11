use crate::id::{Id, IdContext};
use crate::store::Store;

pub trait Traversable<Visitor, Accumulator, S, M, R> {
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        accumulator: &mut Accumulator,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    );

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        accumulator: &mut Accumulator,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool;
}

pub trait Visitor<Node, S, M, R> {
    type Accumulator;

    fn visit(
        &mut self,
        node: &mut Node,
        accumulator: &mut Self::Accumulator,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    );
}
