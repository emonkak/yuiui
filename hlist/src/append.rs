use crate::tuple::{IntoHList, IntoTuple};

use super::{HCons, HList, HNil};

pub trait Append<T> {
    type Output;

    fn append(self, rhs: T) -> Self::Output;
}

impl<T> Append<T> for HNil {
    type Output = T;

    #[inline]
    fn append(self, rhs: T) -> Self::Output {
        rhs
    }
}

impl<Head, Tail, T> Append<T> for HCons<Head, Tail>
where
    Tail: Append<T> + HList,
    Tail::Output: HList,
{
    type Output = HCons<Head, Tail::Output>;

    #[inline]
    fn append(self, rhs: T) -> Self::Output {
        HCons {
            head: self.head,
            tail: self.tail.append(rhs),
        }
    }
}

impl<T, U> Append<U> for T
where
    T: IntoHList,
    T::IntoHList: Append<U::IntoHList>,
    <T::IntoHList as Append<U::IntoHList>>::Output: IntoTuple,
    U: IntoHList,
{
    type Output = <<T::IntoHList as Append<U::IntoHList>>::Output as IntoTuple>::IntoTuple;

    #[inline]
    fn append(self, rhs: U) -> Self::Output {
        self.into_hlist().append(rhs.into_hlist()).into_tuple()
    }
}
