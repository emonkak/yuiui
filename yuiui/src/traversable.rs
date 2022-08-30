use crate::id::IdPath;

pub trait Traversable<Visitor, Context, S, E> {
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

pub trait TraversableVisitor<Node, Context, S, E> {
    fn visit(&mut self, node: &mut Node, state: &S, env: &E, context: &mut Context);
}
