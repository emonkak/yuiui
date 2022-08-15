use std::marker::PhantomData;

pub trait Nat {
    const N: usize;
}

pub enum Zero {}

impl Nat for Zero {
    const N: usize = 0;
}

pub struct Succ<T: Nat>(PhantomData<T>);

impl<T: Nat> Nat for Succ<T> {
    const N: usize = 1 + T::N;
}

pub trait Sub<Rhs> {
    type Output;
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
