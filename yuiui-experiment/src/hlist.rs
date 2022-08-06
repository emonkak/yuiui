use std::convert::Infallible;
use std::fmt;

pub trait HList: Sized + private::Sealed {
    type Head;

    type Tail: HList;

    fn construct<V>(self, value: V) -> HCons<V, Self>;

    fn destruct(self) -> Option<(Self::Head, Self::Tail)>;
}

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HNil;

impl HList for HNil {
    type Head = Infallible;

    type Tail = HNil;

    fn construct<V>(self, value: V) -> HCons<V, Self> {
        HCons(value, self)
    }

    fn destruct(self) -> Option<(Self::Head, Self::Tail)> {
        None
    }
}

impl fmt::Debug for HNil {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("hlist![]")
    }
}

impl private::Sealed for HNil {}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HCons<H, T: HList>(pub H, pub T);

impl<H, T: HList> HList for HCons<H, T> {
    type Head = H;

    type Tail = T;

    fn construct<V>(self, value: V) -> HCons<V, Self> {
        HCons(value, self)
    }

    fn destruct(self) -> Option<(Self::Head, Self::Tail)> {
        Some((self.0, self.1))
    }
}

impl<H: fmt::Debug, T: HList + DebugList> fmt::Debug for HCons<H, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("hlist!")?;
        DebugList::fmt(self, &mut f.debug_list())
    }
}

impl<H, T: HList> private::Sealed for HCons<H, T> {}

trait DebugList {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result;
}

impl<H: fmt::Debug, T: HList + DebugList> DebugList for HCons<H, T> {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result {
        debug_list.entry(&self.0);
        self.1.fmt(debug_list)
    }
}

impl DebugList for HNil {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result {
        debug_list.finish()
    }
}

mod private {
    pub trait Sealed {}
}

#[macro_export]
macro_rules! hlist {
    () => { $crate::hlist::HNil };
    ($head:expr) => { $crate::hlist![$head,] };
    ($head:expr, $($tail:tt)*) => {
        $crate::hlist::HCons($head, $crate::hlist![$($tail)*])
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
