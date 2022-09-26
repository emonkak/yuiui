mod debug;
mod hlist;
mod index;
mod nat;

pub use self::hlist::{HCons, HList, HNil};
pub use self::index::{Index, LastIndex};
pub use self::nat::{Nat, Succ, Zero};

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

    fn index<N, T: Index<U, N>, U>(value: &T) -> &U {
        value.index()
    }

    fn last_index<N, T: LastIndex<U, N>, U>(value: &T) -> &U {
        value.last_index()
    }
}
