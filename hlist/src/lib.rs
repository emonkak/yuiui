pub mod nat;

mod debug;
mod first;
mod last;

pub use first::First;
pub use last::Last;

use nat::{Nat, Succ, Zero};
use std::fmt;

use debug::DebugHList;

mod private {
    pub trait Sealed {}
}

pub trait HList: Sized + private::Sealed {
    type Len: Nat;

    const LEN: usize;

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

    const LEN: usize = 1 + Tail::LEN;
}

impl<Head, Tail> fmt::Debug for HCons<Head, Tail>
where
    Head: fmt::Debug,
    Tail: DebugHList,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("hlist!")?;
        DebugHList::fmt(self, &mut f.debug_list())
    }
}

impl<Head, Tail: HList> private::Sealed for HCons<Head, Tail> {}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HNil;

impl HList for HNil {
    type Len = Zero;

    const LEN: usize = 0;
}

impl fmt::Debug for HNil {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("hlist![]")
    }
}

impl private::Sealed for HNil {}

#[macro_export]
macro_rules! hlist {
    () => { ::hlist::HNil };
    ($head:expr) => { ::hlist::hlist![$head,] };
    ($head:expr, $($tail:tt)*) => {
        $crate::HCons { head: $head, tail: ::hlist::hlist![$($tail)*] }
    };
}

#[macro_export]
macro_rules! HList {
    () => { ::hlist::HNil };
    ($head:ty) => { ::hlist::HList![$head,] };
    ($head:ty, $($tail:tt)*) => {
        ::hlist::HCons<$head, ::hlist::HList![$($tail)*]>
    };
}
