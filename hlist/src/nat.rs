use std::marker::PhantomData;

pub trait Nat {
    const N: usize;
}

pub type N0 = Zero;
pub type N1 = Succ<N0>;
pub type N2 = Succ<N1>;
pub type N3 = Succ<N2>;
pub type N4 = Succ<N3>;
pub type N5 = Succ<N4>;
pub type N6 = Succ<N5>;
pub type N7 = Succ<N6>;
pub type N8 = Succ<N7>;
pub type N9 = Succ<N8>;
pub type N10 = Succ<N9>;

pub enum Zero {}

impl Nat for Zero {
    const N: usize = 0;
}

pub struct Succ<T: Nat>(PhantomData<T>);

impl<T: Nat> Nat for Succ<T> {
    const N: usize = 1 + T::N;
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
