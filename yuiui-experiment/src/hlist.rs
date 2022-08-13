use std::fmt;

pub trait HList: Sized + private::Sealed {}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HCons<H, T: HList>(pub H, pub T);

impl<H, T: HList> HList for HCons<H, T> {}

impl<H: fmt::Debug, T: HList + DebugHList> fmt::Debug for HCons<H, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("hlist!")?;
        DebugHList::fmt(self, &mut f.debug_list())
    }
}

impl<H, T: HList> private::Sealed for HCons<H, T> {}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HNil;

impl HList for HNil {}

impl fmt::Debug for HNil {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("hlist![]")
    }
}

impl private::Sealed for HNil {}

trait DebugHList {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result;
}

impl<H: fmt::Debug, T: HList + DebugHList> DebugHList for HCons<H, T> {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result {
        debug_list.entry(&self.0);
        self.1.fmt(debug_list)
    }
}

impl DebugHList for HNil {
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
macro_rules! HList {
    () => { $crate::hlist::HNil };
    ($head:ty) => { $crate::HList![$head,] };
    ($head:ty, $($tail:tt)*) => {
        $crate::hlist::HCons<$head, $crate::HList![$($tail)*]>
    };
}
