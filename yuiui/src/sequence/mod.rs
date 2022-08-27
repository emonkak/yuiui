mod array;
mod either;
mod hlist;
mod option;
mod vec;

use crate::id::IdPath;
use crate::state::State;

pub trait TraversableSeq<Visitor, Context, S: State, E> {
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut Context);

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool;
}

pub trait TraversableSeqVisitor<Node, Context, S: State, E> {
    fn visit(&mut self, node: &mut Node, state: &S, env: &E, context: &mut Context);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RenderStatus {
    Unchanged,
    Changed,
    Swapped,
}
