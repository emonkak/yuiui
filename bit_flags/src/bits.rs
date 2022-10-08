use std::ops;

pub trait IntoBits {
    type Bits: Bits;

    fn into_bits(self) -> Self::Bits;
}

pub trait Bits:
    Clone
    + Copy
    + Eq
    + PartialEq
    + Sized
    + ops::BitAnd<Output = Self>
    + ops::BitAndAssign
    + ops::BitOr<Output = Self>
    + ops::BitOrAssign
    + ops::BitXor<Output = Self>
    + ops::BitXorAssign
    + ops::Not<Output = Self>
    + ops::Shl<u32, Output = Self>
    + ops::Sub<Output = Self>
    + ops::SubAssign
{
    const ZERO: Self;

    const ONE: Self;

    fn count_ones(self) -> u32;

    fn trailing_zeros(self) -> u32;

    fn wrapping_sub(self, rhs: Self) -> Self;
}

macro_rules! define_bits_impl {
    ($T:ident) => {
        impl Bits for $T {
            const ZERO: Self = 0;

            const ONE: Self = 1;

            #[inline]
            fn count_ones(self) -> u32 {
                self.count_ones()
            }

            #[inline]
            fn trailing_zeros(self) -> u32 {
                self.trailing_zeros()
            }

            #[inline]
            fn wrapping_sub(self, rhs: Self) -> Self {
                self.wrapping_sub(rhs)
            }
        }
    };
}

define_bits_impl!(u8);
define_bits_impl!(u16);
define_bits_impl!(u32);
define_bits_impl!(u64);
define_bits_impl!(u128);
define_bits_impl!(usize);
