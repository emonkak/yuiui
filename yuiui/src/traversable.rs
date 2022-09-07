use crate::id::IdPath;

pub trait Traversable<Visitor, Context, S, B> {
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> bool;

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> bool;
}

pub trait TraversableVisitor<Node, Context, S, B> {
    fn visit(&mut self, node: &mut Node, state: &S, backend: &B, context: &mut Context) -> bool;
}
