use std::ops::Sub;

use crate::nat::{Nat, Succ, Zero};

use super::{HCons, HList};

pub trait Index<T, N> {
    fn index(&self) -> &T;

    fn index_mut(&mut self) -> &mut T;
}

impl<T, Tail> Index<T, Zero> for HCons<T, Tail>
where
    Tail: HList,
{
    fn index(&self) -> &T {
        &self.head
    }

    fn index_mut(&mut self) -> &mut T {
        &mut self.head
    }
}

impl<T, N, Head, Tail> Index<T, Succ<N>> for HCons<Head, Tail>
where
    N: Nat,
    Tail: Index<T, N> + HList,
{
    fn index(&self) -> &T {
        self.tail.index()
    }

    fn index_mut(&mut self) -> &mut T {
        self.tail.index_mut()
    }
}

pub trait LastIndex<T, N> {
    fn last_index(&self) -> &T;

    fn last_index_mut(&mut self) -> &mut T;
}

impl<T, N, Head, Tail> LastIndex<T, N> for HCons<Head, Tail>
where
    N: Nat,
    Tail: HList,
    Tail::Len: Sub<N>,
    <Tail::Len as Sub<N>>::Output: Nat,
    Self: Index<T, <<Self as HList>::Len as Sub<Succ<N>>>::Output>,
{
    fn last_index(&self) -> &T {
        self.index()
    }

    fn last_index_mut(&mut self) -> &mut T {
        self.index_mut()
    }
}
