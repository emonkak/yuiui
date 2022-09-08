use crate::id::IdPath;

pub trait Traversable<Visitor, Context, Output, S, B> {
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Output;

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Option<Output>;
}

pub trait Visitor<Node, Context, S, B> {
    type Output: Monoid;

    fn visit(
        &mut self,
        node: &mut Node,
        context: &mut Context,
        state: &S,
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
