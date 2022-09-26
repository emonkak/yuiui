use std::marker::PhantomData;

pub trait Nat {
    const VALUE: usize;
}

pub enum Zero {}

impl Nat for Zero {
    const VALUE: usize = 0;
}

pub struct Succ<T: Nat>(PhantomData<T>);

impl<T: Nat> Nat for Succ<T> {
    const VALUE: usize = 1 + T::VALUE;
}

pub trait Add<Rhs: Nat> {
    type Output: Nat;
}

impl Add<Zero> for Zero {
    type Output = Zero;
}

impl<N: Nat> Add<Succ<N>> for Zero {
    type Output = Succ<N>;
}

impl<N: Nat> Add<Zero> for Succ<N> {
    type Output = Succ<N>;
}

impl<N: Nat + Add<M>, M: Nat> Add<Succ<M>> for Succ<N> {
    type Output = Succ<Succ<<N as Add<M>>::Output>>;
}

pub trait Sub<Rhs: Nat> {
    type Output: Nat;
}

impl Sub<Zero> for Zero {
    type Output = Zero;
}

impl<N: Nat> Sub<Zero> for Succ<N> {
    type Output = Succ<N>;
}

impl<N: Nat + Sub<M>, M: Nat> Sub<Succ<M>> for Succ<N> {
    type Output = <N as Sub<M>>::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    type N0 = Zero;
    type N1 = Succ<N0>;
    type N2 = Succ<N1>;
    type N3 = Succ<N2>;
    type N4 = Succ<N3>;
    type N5 = Succ<N4>;
    type N6 = Succ<N5>;
    type N7 = Succ<N6>;
    type N8 = Succ<N7>;
    type N9 = Succ<N8>;
    type N10 = Succ<N9>;

    #[test]
    fn test_nat() {
        assert_eq!(N0::VALUE, 0);
        assert_eq!(N1::VALUE, 1);
        assert_eq!(N2::VALUE, 2);
        assert_eq!(N3::VALUE, 3);
        assert_eq!(N4::VALUE, 4);
        assert_eq!(N5::VALUE, 5);
        assert_eq!(N6::VALUE, 6);
        assert_eq!(N7::VALUE, 7);
        assert_eq!(N8::VALUE, 8);
        assert_eq!(N9::VALUE, 9);
        assert_eq!(N10::VALUE, 10);
    }

    #[test]
    fn test_add() {
        assert_eq!(<N0 as Add<N0>>::Output::VALUE, 0);
        assert_eq!(<N0 as Add<N1>>::Output::VALUE, 1);
        assert_eq!(<N0 as Add<N2>>::Output::VALUE, 2);
        assert_eq!(<N1 as Add<N0>>::Output::VALUE, 1);
        assert_eq!(<N1 as Add<N1>>::Output::VALUE, 2);
        assert_eq!(<N1 as Add<N2>>::Output::VALUE, 3);
        assert_eq!(<N2 as Add<N0>>::Output::VALUE, 2);
        assert_eq!(<N2 as Add<N1>>::Output::VALUE, 3);
        assert_eq!(<N2 as Add<N2>>::Output::VALUE, 4);
    }

    #[test]
    fn test_sub() {
        assert_eq!(<N0 as Sub<N0>>::Output::VALUE, 0);
        assert_eq!(<N1 as Sub<N0>>::Output::VALUE, 1);
        assert_eq!(<N1 as Sub<N1>>::Output::VALUE, 0);
        assert_eq!(<N2 as Sub<N0>>::Output::VALUE, 2);
        assert_eq!(<N2 as Sub<N1>>::Output::VALUE, 1);
        assert_eq!(<N2 as Sub<N2>>::Output::VALUE, 0);
    }
}
