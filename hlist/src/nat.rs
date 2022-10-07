use std::ops::{Add, Div, Mul, Rem, Sub};

pub trait Nat: Copy + private::Sealed {
    const VALUE: usize;
}

#[derive(Clone, Copy, Debug)]
pub struct Zero;

#[derive(Clone, Copy, Debug)]
pub struct Succ<T: Nat>(T);

impl Nat for Zero {
    const VALUE: usize = 0;
}

impl private::Sealed for Zero {}

impl<T: Nat> Nat for Succ<T> {
    const VALUE: usize = 1 + T::VALUE;
}

impl<T: Nat> private::Sealed for Succ<T> {}

pub trait Compare<T> {
    type Output;
}

#[derive(Clone, Copy, Debug)]
pub struct Less;

#[derive(Clone, Copy, Debug)]
pub struct Equal;

#[derive(Clone, Copy, Debug)]
pub struct Greater;

impl Compare<Zero> for Zero {
    type Output = Equal;
}

impl<M: Nat> Compare<Succ<M>> for Zero {
    type Output = Less;
}

impl<N: Nat> Compare<Zero> for Succ<N> {
    type Output = Greater;
}

impl<N, M> Compare<Succ<M>> for Succ<N>
where
    N: Nat + Compare<M>,
    M: Nat,
{
    type Output = <N as Compare<M>>::Output;
}

impl<N: Nat> Add<N> for Zero {
    type Output = N;

    #[inline]
    fn add(self, rhs: N) -> Self::Output {
        rhs
    }
}

impl<N: Nat> Add<Zero> for Succ<N> {
    type Output = Succ<N>;

    #[inline]
    fn add(self, _rhs: Zero) -> Self::Output {
        self
    }
}

impl<N, M> Add<Succ<M>> for Succ<N>
where
    N: Nat + Add<M>,
    N::Output: Nat,
    M: Nat,
{
    type Output = Succ<Succ<<N as Add<M>>::Output>>;

    #[inline]
    fn add(self, rhs: Succ<M>) -> Self::Output {
        Succ(Succ(self.0 + rhs.0))
    }
}

impl Sub<Zero> for Zero {
    type Output = Zero;

    #[inline]
    fn sub(self, _rhs: Zero) -> Self::Output {
        self
    }
}

impl<N: Nat> Sub<Zero> for Succ<N> {
    type Output = Succ<N>;

    #[inline]
    fn sub(self, _rhs: Zero) -> Self::Output {
        self
    }
}

impl<N, M> Sub<Succ<M>> for Succ<N>
where
    N: Nat + Sub<M>,
    N::Output: Nat,
    M: Nat,
{
    type Output = <N as Sub<M>>::Output;

    #[inline]
    fn sub(self, rhs: Succ<M>) -> Self::Output {
        self.0 - rhs.0
    }
}

impl<M: Nat> Mul<M> for Zero {
    type Output = Zero;

    #[inline]
    fn mul(self, _rhs: M) -> Self::Output {
        self
    }
}

impl<N, M> Mul<M> for Succ<N>
where
    N: Nat + Mul<M>,
    M: Nat + Add<<N as Mul<M>>::Output>,
{
    type Output = <M as Add<<N as Mul<M>>::Output>>::Output;

    #[inline]
    fn mul(self, rhs: M) -> Self::Output {
        rhs + (self.0 * rhs)
    }
}

impl<M: Nat> Div<Succ<M>> for Zero {
    type Output = Zero;

    #[inline]
    fn div(self, _rhs: Succ<M>) -> Self::Output {
        self
    }
}

impl<N, M> Div<Succ<M>> for Succ<N>
where
    N: Nat,
    M: Nat,
    Succ<N>: Compare<Succ<M>> + private::Div<Succ<M>, <Succ<N> as Compare<Succ<M>>>::Output>,
{
    // N % M = (N - M) % M
    type Output = <Succ<N> as private::Div<Succ<M>, <Succ<N> as Compare<Succ<M>>>::Output>>::Output;

    #[inline]
    fn div(self, rhs: Succ<M>) -> Self::Output {
        private::Div::div(self, rhs)
    }
}

impl<N: Nat> Rem<Succ<N>> for Zero {
    type Output = Zero;

    #[inline]
    fn rem(self, _rhs: Succ<N>) -> Self::Output {
        self
    }
}

impl<N, M> Rem<Succ<M>> for Succ<N>
where
    N: Nat,
    M: Nat,
    Succ<N>: Compare<Succ<M>> + private::Rem<Succ<M>, <Succ<N> as Compare<Succ<M>>>::Output>,
{
    // N % M = (N - M) % M
    type Output = <Succ<N> as private::Rem<Succ<M>, <Succ<N> as Compare<Succ<M>>>::Output>>::Output;

    #[inline]
    fn rem(self, rhs: Succ<M>) -> Self::Output {
        private::Rem::rem(self, rhs)
    }
}

mod private {
    use std::ops;

    use super::{Equal, Greater, Less, Nat, Succ, Zero};

    pub trait Sealed {}

    pub trait Div<T, U> {
        type Output;

        fn div(self, _rhs: T) -> Self::Output;
    }

    impl<N: Nat, M: Nat> Div<Succ<M>, Less> for Succ<N> {
        type Output = Zero;

        #[inline]
        fn div(self, _rhs: Succ<M>) -> Self::Output {
            Zero
        }
    }

    impl<N: Nat, M: Nat> Div<Succ<M>, Equal> for Succ<N> {
        type Output = Succ<Zero>;

        #[inline]
        fn div(self, _rhs: Succ<M>) -> Self::Output {
            Succ(Zero)
        }
    }

    impl<N, M> Div<Succ<M>, Greater> for Succ<N>
    where
        N: Nat,
        M: Nat,
        Succ<N>: ops::Sub<Succ<M>>,
        <Succ<N> as ops::Sub<Succ<M>>>::Output: ops::Div<Succ<M>>,
        Succ<Zero>: ops::Add<<<Succ<N> as ops::Sub<Succ<M>>>::Output as ops::Div<Succ<M>>>::Output>,
    {
        // N / M = 1 + (N - M) / M
        type Output = <Succ<Zero> as ops::Add<
            <<Succ<N> as ops::Sub<Succ<M>>>::Output as ops::Div<Succ<M>>>::Output,
        >>::Output;

        #[inline]
        fn div(self, rhs: Succ<M>) -> Self::Output {
            Succ(Zero) + (self - rhs) / rhs
        }
    }

    pub trait Rem<T, U> {
        type Output;

        fn rem(self, _rhs: T) -> Self::Output;
    }

    impl<N: Nat, M: Nat> Rem<Succ<M>, Less> for Succ<N> {
        type Output = Succ<N>;

        #[inline]
        fn rem(self, _rhs: Succ<M>) -> Self::Output {
            self
        }
    }

    impl<N: Nat, M: Nat> Rem<Succ<M>, Equal> for Succ<N> {
        type Output = Zero;

        #[inline]
        fn rem(self, _rhs: Succ<M>) -> Self::Output {
            Zero
        }
    }

    impl<N, M> Rem<Succ<M>, Greater> for Succ<N>
    where
        N: Nat,
        M: Nat,
        Succ<N>: ops::Sub<Succ<M>>,
        <Succ<N> as ops::Sub<Succ<M>>>::Output: ops::Rem<Succ<M>>,
    {
        // N % M = (N - M) % M
        type Output = <<Succ<N> as ops::Sub<Succ<M>>>::Output as ops::Rem<Succ<M>>>::Output;

        #[inline]
        fn rem(self, rhs: Succ<M>) -> Self::Output {
            (self - rhs) % rhs
        }
    }
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
        assert_eq!(<N0 as Add<N3>>::Output::VALUE, 3);
        assert_eq!(<N0 as Add<N4>>::Output::VALUE, 4);
        assert_eq!(<N0 as Add<N5>>::Output::VALUE, 5);

        assert_eq!(<N1 as Add<N0>>::Output::VALUE, 1);
        assert_eq!(<N1 as Add<N1>>::Output::VALUE, 2);
        assert_eq!(<N1 as Add<N2>>::Output::VALUE, 3);
        assert_eq!(<N1 as Add<N3>>::Output::VALUE, 4);
        assert_eq!(<N1 as Add<N4>>::Output::VALUE, 5);
        assert_eq!(<N1 as Add<N5>>::Output::VALUE, 6);

        assert_eq!(<N2 as Add<N0>>::Output::VALUE, 2);
        assert_eq!(<N2 as Add<N1>>::Output::VALUE, 3);
        assert_eq!(<N2 as Add<N2>>::Output::VALUE, 4);
        assert_eq!(<N2 as Add<N3>>::Output::VALUE, 5);
        assert_eq!(<N2 as Add<N4>>::Output::VALUE, 6);
        assert_eq!(<N2 as Add<N5>>::Output::VALUE, 7);

        assert_eq!(<N3 as Add<N0>>::Output::VALUE, 3);
        assert_eq!(<N3 as Add<N1>>::Output::VALUE, 4);
        assert_eq!(<N3 as Add<N2>>::Output::VALUE, 5);
        assert_eq!(<N3 as Add<N3>>::Output::VALUE, 6);
        assert_eq!(<N3 as Add<N4>>::Output::VALUE, 7);
        assert_eq!(<N3 as Add<N5>>::Output::VALUE, 8);

        assert_eq!(<N4 as Add<N0>>::Output::VALUE, 4);
        assert_eq!(<N4 as Add<N1>>::Output::VALUE, 5);
        assert_eq!(<N4 as Add<N2>>::Output::VALUE, 6);
        assert_eq!(<N4 as Add<N3>>::Output::VALUE, 7);
        assert_eq!(<N4 as Add<N4>>::Output::VALUE, 8);
        assert_eq!(<N4 as Add<N5>>::Output::VALUE, 9);

        assert_eq!(<N5 as Add<N0>>::Output::VALUE, 5);
        assert_eq!(<N5 as Add<N1>>::Output::VALUE, 6);
        assert_eq!(<N5 as Add<N2>>::Output::VALUE, 7);
        assert_eq!(<N5 as Add<N3>>::Output::VALUE, 8);
        assert_eq!(<N5 as Add<N4>>::Output::VALUE, 9);
        assert_eq!(<N5 as Add<N5>>::Output::VALUE, 10);
    }

    #[test]
    fn test_sub() {
        assert_eq!(<N0 as Sub<N0>>::Output::VALUE, 0);

        assert_eq!(<N1 as Sub<N0>>::Output::VALUE, 1);
        assert_eq!(<N1 as Sub<N1>>::Output::VALUE, 0);

        assert_eq!(<N2 as Sub<N0>>::Output::VALUE, 2);
        assert_eq!(<N2 as Sub<N1>>::Output::VALUE, 1);
        assert_eq!(<N2 as Sub<N2>>::Output::VALUE, 0);

        assert_eq!(<N3 as Sub<N0>>::Output::VALUE, 3);
        assert_eq!(<N3 as Sub<N1>>::Output::VALUE, 2);
        assert_eq!(<N3 as Sub<N2>>::Output::VALUE, 1);
        assert_eq!(<N3 as Sub<N3>>::Output::VALUE, 0);

        assert_eq!(<N4 as Sub<N0>>::Output::VALUE, 4);
        assert_eq!(<N4 as Sub<N1>>::Output::VALUE, 3);
        assert_eq!(<N4 as Sub<N2>>::Output::VALUE, 2);
        assert_eq!(<N4 as Sub<N3>>::Output::VALUE, 1);
        assert_eq!(<N4 as Sub<N4>>::Output::VALUE, 0);

        assert_eq!(<N5 as Sub<N0>>::Output::VALUE, 5);
        assert_eq!(<N5 as Sub<N1>>::Output::VALUE, 4);
        assert_eq!(<N5 as Sub<N2>>::Output::VALUE, 3);
        assert_eq!(<N5 as Sub<N3>>::Output::VALUE, 2);
        assert_eq!(<N5 as Sub<N4>>::Output::VALUE, 1);
        assert_eq!(<N5 as Sub<N5>>::Output::VALUE, 0);
    }

    #[test]
    fn test_mut() {
        assert_eq!(<N0 as Mul<N0>>::Output::VALUE, 0);
        assert_eq!(<N0 as Mul<N1>>::Output::VALUE, 0);
        assert_eq!(<N0 as Mul<N2>>::Output::VALUE, 0);
        assert_eq!(<N0 as Mul<N3>>::Output::VALUE, 0);
        assert_eq!(<N0 as Mul<N4>>::Output::VALUE, 0);
        assert_eq!(<N0 as Mul<N5>>::Output::VALUE, 0);

        assert_eq!(<N1 as Mul<N0>>::Output::VALUE, 0);
        assert_eq!(<N1 as Mul<N1>>::Output::VALUE, 1);
        assert_eq!(<N1 as Mul<N2>>::Output::VALUE, 2);
        assert_eq!(<N1 as Mul<N3>>::Output::VALUE, 3);
        assert_eq!(<N1 as Mul<N4>>::Output::VALUE, 4);
        assert_eq!(<N1 as Mul<N5>>::Output::VALUE, 5);

        assert_eq!(<N2 as Mul<N0>>::Output::VALUE, 0);
        assert_eq!(<N2 as Mul<N1>>::Output::VALUE, 2);
        assert_eq!(<N2 as Mul<N2>>::Output::VALUE, 4);
        assert_eq!(<N2 as Mul<N3>>::Output::VALUE, 6);
        assert_eq!(<N2 as Mul<N4>>::Output::VALUE, 8);
        assert_eq!(<N2 as Mul<N5>>::Output::VALUE, 10);

        assert_eq!(<N3 as Mul<N0>>::Output::VALUE, 0);
        assert_eq!(<N3 as Mul<N1>>::Output::VALUE, 3);
        assert_eq!(<N3 as Mul<N2>>::Output::VALUE, 6);
        assert_eq!(<N3 as Mul<N3>>::Output::VALUE, 9);
        assert_eq!(<N3 as Mul<N4>>::Output::VALUE, 12);
        assert_eq!(<N3 as Mul<N5>>::Output::VALUE, 15);

        assert_eq!(<N4 as Mul<N0>>::Output::VALUE, 0);
        assert_eq!(<N4 as Mul<N1>>::Output::VALUE, 4);
        assert_eq!(<N4 as Mul<N2>>::Output::VALUE, 8);
        assert_eq!(<N4 as Mul<N3>>::Output::VALUE, 12);
        assert_eq!(<N4 as Mul<N4>>::Output::VALUE, 16);
        assert_eq!(<N4 as Mul<N5>>::Output::VALUE, 20);

        assert_eq!(<N5 as Mul<N0>>::Output::VALUE, 0);
        assert_eq!(<N5 as Mul<N1>>::Output::VALUE, 5);
        assert_eq!(<N5 as Mul<N2>>::Output::VALUE, 10);
        assert_eq!(<N5 as Mul<N3>>::Output::VALUE, 15);
        assert_eq!(<N5 as Mul<N4>>::Output::VALUE, 20);
        assert_eq!(<N5 as Mul<N5>>::Output::VALUE, 25);
    }

    #[test]
    fn test_div() {
        assert_eq!(<N0 as Div<N1>>::Output::VALUE, 0);
        assert_eq!(<N0 as Div<N2>>::Output::VALUE, 0);
        assert_eq!(<N0 as Div<N3>>::Output::VALUE, 0);
        assert_eq!(<N0 as Div<N4>>::Output::VALUE, 0);
        assert_eq!(<N0 as Div<N5>>::Output::VALUE, 0);

        assert_eq!(<N1 as Div<N1>>::Output::VALUE, 1);
        assert_eq!(<N1 as Div<N2>>::Output::VALUE, 0);
        assert_eq!(<N1 as Div<N3>>::Output::VALUE, 0);
        assert_eq!(<N1 as Div<N4>>::Output::VALUE, 0);
        assert_eq!(<N1 as Div<N5>>::Output::VALUE, 0);

        assert_eq!(<N2 as Div<N1>>::Output::VALUE, 2);
        assert_eq!(<N2 as Div<N2>>::Output::VALUE, 1);
        assert_eq!(<N2 as Div<N3>>::Output::VALUE, 0);
        assert_eq!(<N2 as Div<N4>>::Output::VALUE, 0);
        assert_eq!(<N2 as Div<N5>>::Output::VALUE, 0);

        assert_eq!(<N3 as Div<N1>>::Output::VALUE, 3);
        assert_eq!(<N3 as Div<N2>>::Output::VALUE, 1);
        assert_eq!(<N3 as Div<N3>>::Output::VALUE, 1);
        assert_eq!(<N3 as Div<N4>>::Output::VALUE, 0);
        assert_eq!(<N3 as Div<N5>>::Output::VALUE, 0);

        assert_eq!(<N4 as Div<N1>>::Output::VALUE, 4);
        assert_eq!(<N4 as Div<N2>>::Output::VALUE, 2);
        assert_eq!(<N4 as Div<N3>>::Output::VALUE, 1);
        assert_eq!(<N4 as Div<N4>>::Output::VALUE, 1);
        assert_eq!(<N4 as Div<N5>>::Output::VALUE, 0);

        assert_eq!(<N5 as Div<N1>>::Output::VALUE, 5);
        assert_eq!(<N5 as Div<N2>>::Output::VALUE, 2);
        assert_eq!(<N5 as Div<N3>>::Output::VALUE, 1);
        assert_eq!(<N5 as Div<N5>>::Output::VALUE, 1);
    }

    #[test]
    fn test_rem() {
        assert_eq!(<N0 as Rem<N1>>::Output::VALUE, 0);
        assert_eq!(<N0 as Rem<N2>>::Output::VALUE, 0);
        assert_eq!(<N0 as Rem<N3>>::Output::VALUE, 0);
        assert_eq!(<N0 as Rem<N4>>::Output::VALUE, 0);
        assert_eq!(<N0 as Rem<N5>>::Output::VALUE, 0);

        assert_eq!(<N1 as Rem<N1>>::Output::VALUE, 0);
        assert_eq!(<N1 as Rem<N2>>::Output::VALUE, 1);
        assert_eq!(<N1 as Rem<N3>>::Output::VALUE, 1);
        assert_eq!(<N1 as Rem<N4>>::Output::VALUE, 1);
        assert_eq!(<N1 as Rem<N5>>::Output::VALUE, 1);

        assert_eq!(<N2 as Rem<N1>>::Output::VALUE, 0);
        assert_eq!(<N2 as Rem<N2>>::Output::VALUE, 0);
        assert_eq!(<N2 as Rem<N3>>::Output::VALUE, 2);
        assert_eq!(<N2 as Rem<N4>>::Output::VALUE, 2);
        assert_eq!(<N2 as Rem<N5>>::Output::VALUE, 2);

        assert_eq!(<N3 as Rem<N1>>::Output::VALUE, 0);
        assert_eq!(<N3 as Rem<N2>>::Output::VALUE, 1);
        assert_eq!(<N3 as Rem<N3>>::Output::VALUE, 0);
        assert_eq!(<N3 as Rem<N4>>::Output::VALUE, 3);
        assert_eq!(<N3 as Rem<N5>>::Output::VALUE, 3);

        assert_eq!(<N4 as Rem<N1>>::Output::VALUE, 0);
        assert_eq!(<N4 as Rem<N2>>::Output::VALUE, 0);
        assert_eq!(<N4 as Rem<N3>>::Output::VALUE, 1);
        assert_eq!(<N4 as Rem<N4>>::Output::VALUE, 0);
        assert_eq!(<N4 as Rem<N5>>::Output::VALUE, 4);

        assert_eq!(<N5 as Rem<N1>>::Output::VALUE, 0);
        assert_eq!(<N5 as Rem<N2>>::Output::VALUE, 1);
        assert_eq!(<N5 as Rem<N3>>::Output::VALUE, 2);
        assert_eq!(<N5 as Rem<N4>>::Output::VALUE, 1);
        assert_eq!(<N5 as Rem<N5>>::Output::VALUE, 0);
    }
}
