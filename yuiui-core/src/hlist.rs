pub trait HList: private::Sealed {}

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HNil;

impl HList for HNil {}

impl private::Sealed for HNil {}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HCons<Head, Tail: HList> {
    pub head: Head,
    pub tail: Tail,
}

impl<Head, Tail: HList> HList for HCons<Head, Tail> {}

impl<Head, Tail: HList> private::Sealed for HCons<Head, Tail> {}

mod private {
    pub trait Sealed {}
}

#[macro_export]
macro_rules! hlist {
    () => { $crate::HNil };
    ($head:ident) => { $crate::hlist![$head,] };
    ($head:ident, $($tail:tt)*) => {
        $crate::HCons { head: $head, tail: $crate::hlist![$($tail)*] }
    };
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
