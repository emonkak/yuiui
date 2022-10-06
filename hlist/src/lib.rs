mod append;
mod debug;
mod index;
mod nat;
mod tuple;

pub use self::append::Append;
pub use self::index::{Index, LastIndex};
pub use self::nat::{Nat, Succ, Zero};
pub use self::tuple::{IntoHList, IntoTuple};

mod private {
    pub trait Sealed {}
}

pub trait HList: Sized + private::Sealed {
    type Len: Nat;

    fn cons<T>(self, head: T) -> HCons<T, Self> {
        HCons { head, tail: self }
    }
}

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HNil;

impl HList for HNil {
    type Len = Zero;
}

impl private::Sealed for HNil {}

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

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn test_index() {
        let xs = hlist![123, 456.0, true, "foo"];
        assert_eq!(*index::<Zero, _, _>(&xs), 123);
        assert_eq!(*index::<Succ<Zero>, _, _>(&xs), 456.0);
        assert_eq!(*index::<Succ<Succ<Zero>>, _, _>(&xs), true);
        assert_eq!(*index::<Succ<Succ<Succ<Zero>>>, _, _>(&xs), "foo");
    }

    #[test]
    fn test_last_index() {
        let xs = hlist![123, 456.0, true, "foo"];
        assert_eq!(*last_index::<Zero, _, _>(&xs), "foo");
        assert_eq!(*last_index::<Succ<Zero>, _, _>(&xs), true);
        assert_eq!(*last_index::<Succ<Succ<Zero>>, _, _>(&xs), 456.0);
        assert_eq!(*last_index::<Succ<Succ<Succ<Zero>>>, _, _>(&xs), 123);
    }

    #[test]
    fn test_append_hlist() {
        let xs = hlist![123, 456.0].append(hlist![true, "foo"]);
        assert_eq!(xs, hlist![123, 456.0, true, "foo"]);

        let xs = hlist![123, 456.0]
            .append(hlist![])
            .append(hlist![true])
            .append(hlist!["foo"]);
        assert_eq!(xs, hlist![123, 456.0, true, "foo"]);
    }

    #[test]
    fn test_append_tuple() {
        let xs = (123, 456.0).append((true, "foo"));
        assert_eq!(xs, (123, 456.0, true, "foo"));

        let xs = (123, 456.0).append(()).append((true,)).append(("foo",));
        assert_eq!(xs, (123, 456.0, true, "foo"));
    }

    fn index<N, T: Index<U, N>, U>(value: &T) -> &U {
        value.index()
    }

    fn last_index<N, T: LastIndex<U, N>, U>(value: &T) -> &U {
        value.last_index()
    }
}
