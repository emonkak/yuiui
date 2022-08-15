use crate::nat::{Nat, Succ, Zero};

use super::{HCons, HList};

pub trait First<T, Index = Zero> {
    fn first(&self) -> &T;

    fn first_mut(&mut self) -> &mut T;
}

impl<T, Tail> First<T, Zero> for HCons<T, Tail>
where
    Tail: HList,
{
    fn first(&self) -> &T {
        &self.head
    }

    fn first_mut(&mut self) -> &mut T {
        &mut self.head
    }
}

impl<T, Index, Head, Tail> First<T, Succ<Index>> for HCons<Head, Tail>
where
    Index: Nat,
    Tail: First<T, Index> + HList,
{
    fn first(&self) -> &T {
        self.tail.first()
    }

    fn first_mut(&mut self) -> &mut T {
        self.tail.first_mut()
    }
}
