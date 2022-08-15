use super::{HCons, HList};

use crate::first::First;
use crate::nat::{Nat, Sub, Succ, Zero};

pub trait Last<T, Index = Zero> {
    fn last(&self) -> &T;

    fn last_mut(&mut self) -> &mut T;
}

impl<T, Index, Head, Tail> Last<T, Index> for HCons<Head, Tail>
where
    Index: Nat,
    Tail: HList,
    <Tail as HList>::Len: Sub<Index>,
    Self: First<T, <<Self as HList>::Len as Sub<Succ<Index>>>::Output>,
{
    fn last(&self) -> &T {
        self.first()
    }

    fn last_mut(&mut self) -> &mut T {
        self.first_mut()
    }
}
