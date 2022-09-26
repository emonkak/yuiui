use crate::nat::{Nat, Succ, Zero};

mod private {
    pub trait Sealed {}
}

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HNil;

impl HList for HNil {
    type Len = Zero;
}

impl private::Sealed for HNil {}

pub trait HList: Sized + private::Sealed {
    type Len: Nat;

    fn cons<T>(self, head: T) -> HCons<T, Self> {
        HCons { head, tail: self }
    }
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HCons<Head, Tail: HList> {
    pub head: Head,
    pub tail: Tail,
}

impl<Head, Tail: HList> HList for HCons<Head, Tail> {
    type Len = Succ<Tail::Len>;
}

impl<Head, Tail: HList> private::Sealed for HCons<Head, Tail> {}

#[macro_export]
macro_rules! hlist {
    () => { $crate::HNil };
    ($head:expr) => { $crate::hlist![$head,] };
    ($head:expr, $($tail:tt)*) => {
        $crate::HCons { head: $head, tail: $crate::hlist![$($tail)*] }
    };
}

#[macro_export]
macro_rules! HList {
    () => { $crate::HNil };
    ($head:ty) => { $crate::HList![$head,] };
    ($head:ty, $($tail:tt)*) => {
        $crate::HCons<$head, $crate::HList![$($tail)*]>
    };
}
