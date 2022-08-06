use std::convert::Infallible;
use std::fmt;

mod private {
    pub trait Sealed {
    }
}

pub trait HList: Sized + private::Sealed {
    type Head;

    type Tail: HList;

    fn construct<V>(self, value: V) -> HCons<V, Self>;

    fn destruct(self) -> Option<(Self::Head, Self::Tail)>;
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HCons<H, T: HList> {
    pub head: H,
    pub tail: T,
}

impl HList for () {
    type Head = Infallible;

    type Tail = HNil;

    fn construct<V>(self, value: V) -> HCons<V, Self> {
        HCons {
            head: value,
            tail: self,
        }
    }

    fn destruct(self) -> Option<(Self::Head, Self::Tail)> {
        None
    }
}

impl private::Sealed for () {
}

impl<H, T: HList> HList for HCons<H, T> {
    type Head = H;

    type Tail = T;

    fn construct<V>(self, value: V) -> HCons<V, Self> {
        HCons {
            head: value,
            tail: self,
        }
    }

    fn destruct(self) -> Option<(Self::Head, Self::Tail)> {
        Some((self.head, self.tail))
    }
}

impl<H, T: HList> private::Sealed for HCons<H, T> {
}

impl<H: fmt::Debug, T: HList + DebugList> fmt::Debug for HCons<H, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        DebugList::fmt(self, &mut f.debug_list())
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HNil;

impl HList for HNil {
    type Head = Infallible;

    type Tail = HNil;

    fn construct<V>(self, value: V) -> HCons<V, Self> {
        HCons {
            head: value,
            tail: self,
        }
    }

    fn destruct(self) -> Option<(Self::Head, Self::Tail)> {
        None
    }
}

impl private::Sealed for HNil {
}

trait DebugList {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result;
}

impl<H: fmt::Debug, T: HList + DebugList> DebugList for HCons<H, T>
{
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result {
        debug_list.entry(&self.head);
        self.tail.fmt(debug_list)
    }
}

impl DebugList for HNil {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result {
        debug_list.finish()
    }
}

#[macro_export]
macro_rules! hlist {
    () => { $crate::hlist::HNil };
    ($head:expr) => { $crate::hlist![$head,] };
    ($head:expr, $($tail:tt)*) => {
        $crate::hlist::HCons {
            head: $head,
            tail: $crate::hlist![$($tail)*],
        }
    };
}

#[macro_export]
macro_rules! hlist_type {
    () => { $crate::hlist::HNil };
    ($head:ty) => { $crate::hlist_type![$head,] };
    ($head:ty, $($tail:tt)*) => {
        $crate::hlist::HCons<$head, $crate::hlist_type![$($tail)*]>
    };
}
