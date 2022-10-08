pub mod iter;

mod bits;

pub use bits::{Bits, IntoBits};

use iter::Iter;

use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BitFlags<T, B = u32> {
    bits: B,
    _phantom: PhantomData<T>,
}

impl<T, B: Bits> BitFlags<T, B> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            bits: Bits::ZERO,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self) -> B {
        self.bits
    }

    #[inline]
    pub fn set(&mut self, bits: impl IntoBits<Bits = B>) {
        self.bits = bits.into_bits();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits == B::ZERO
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.bits.count_ones()
    }

    #[inline]
    pub fn contains(&self, other: impl Into<Self>) -> bool {
        let bits = other.into().bits;
        (self.bits & bits) == bits
    }

    #[inline]
    pub fn intersects(&self, other: impl Into<Self>) -> bool {
        let bits = other.into().bits;
        (self.bits & bits) != B::ZERO
    }

    #[inline]
    pub fn iter(&self) -> Iter<T, B> {
        Iter::new(self.bits)
    }
}

impl<T, B: Bits> Default for BitFlags<T, B> {
    fn default() -> Self {
        Self {
            bits: B::ZERO,
            _phantom: PhantomData,
        }
    }
}

impl<T: IntoBits<Bits = B>, B> From<T> for BitFlags<T, B> {
    #[inline]
    fn from(value: T) -> Self {
        Self {
            bits: value.into_bits(),
            _phantom: PhantomData,
        }
    }
}

impl<T: IntoBits<Bits = B>, B: Bits, const N: usize> From<[T; N]> for BitFlags<T, B> {
    #[inline]
    fn from(values: [T; N]) -> Self {
        Self {
            bits: values
                .into_iter()
                .fold(B::ZERO, |bits, value| bits | value.into_bits()),
            _phantom: PhantomData,
        }
    }
}

impl<T: IntoBits<Bits = B>, B: Bits> FromIterator<T> for BitFlags<T, B> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        Self {
            bits: values
                .into_iter()
                .fold(B::ZERO, |bits, value| bits | value.into_bits()),
            _phantom: PhantomData,
        }
    }
}

impl<T, B: Bits, Rhs: Into<Self>> ops::BitAnd<Rhs> for BitFlags<T, B> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Rhs) -> Self::Output {
        Self {
            bits: self.bits & rhs.into().bits,
            _phantom: PhantomData,
        }
    }
}

impl<T, B: Bits, Rhs: Into<Self>> ops::BitAndAssign<Rhs> for BitFlags<T, B> {
    #[inline]
    fn bitand_assign(&mut self, rhs: Rhs) {
        self.bits &= rhs.into().bits;
    }
}

impl<T, B: Bits, Rhs: Into<Self>> ops::BitOr<Rhs> for BitFlags<T, B> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Rhs) -> Self::Output {
        Self {
            bits: self.bits | rhs.into().bits,
            _phantom: PhantomData,
        }
    }
}

impl<T, B: Bits, Rhs: Into<Self>> ops::BitOrAssign<Rhs> for BitFlags<T, B> {
    #[inline]
    fn bitor_assign(&mut self, rhs: Rhs) {
        self.bits |= rhs.into().bits;
    }
}

impl<T, B: Bits, Rhs: Into<Self>> ops::BitXor<Rhs> for BitFlags<T, B> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Rhs) -> Self::Output {
        Self {
            bits: self.bits ^ rhs.into().bits,
            _phantom: PhantomData,
        }
    }
}

impl<T, B: Bits, Rhs: Into<Self>> ops::BitXorAssign<Rhs> for BitFlags<T, B> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Rhs) {
        self.bits ^= rhs.into().bits;
    }
}

impl<T, B: Bits, Rhs: Into<Self>> ops::Sub<Rhs> for BitFlags<T, B> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Rhs) -> Self::Output {
        Self {
            bits: self.bits & !rhs.into().bits,
            _phantom: PhantomData,
        }
    }
}

impl<T, B: Bits, Rhs: Into<Self>> ops::SubAssign<Rhs> for BitFlags<T, B> {
    #[inline]
    fn sub_assign(&mut self, rhs: Rhs) {
        self.bits &= !rhs.into().bits;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(u32)]
    enum Button {
        None = 0b000,
        Left = 0b001,
        Right = 0b010,
        Middle = 0b100,
    }

    impl IntoBits for Button {
        type Bits = u32;

        fn into_bits(self) -> Self::Bits {
            self as u32
        }
    }

    #[test]
    fn test_is_empty() {
        assert_eq!((BitFlags::new() as BitFlags<Button>).is_empty(), true);
        assert_eq!((BitFlags::from([]) as BitFlags<Button>).is_empty(), true);
        assert_eq!((BitFlags::from(Button::None)).is_empty(), true);
        assert_eq!((BitFlags::from(Button::Left)).is_empty(), false);
        assert_eq!(
            (BitFlags::from([Button::Left, Button::Right])).is_empty(),
            false
        );
        assert_eq!(
            (BitFlags::from([Button::Left, Button::Right, Button::Middle])).is_empty(),
            false
        );
    }

    #[test]
    fn test_len() {
        assert_eq!((BitFlags::new() as BitFlags<Button>).len(), 0);
        assert_eq!((BitFlags::from([]) as BitFlags<Button>).len(), 0);
        assert_eq!((BitFlags::from([Button::None])).len(), 0);
        assert_eq!((BitFlags::from([Button::Left])).len(), 1);
        assert_eq!((BitFlags::from([Button::Left, Button::Right])).len(), 2);
        assert_eq!(
            (BitFlags::from([Button::Left, Button::Right, Button::Middle])).len(),
            3
        );
    }

    #[test]
    fn test_contains() {
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right, Button::Middle]).contains([]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right]).contains([]),
            true
        );
        assert_eq!(BitFlags::from([Button::Left]).contains([]), true);
        assert_eq!(BitFlags::from([] as [Button; 0]).contains([]), true);

        assert_eq!(
            BitFlags::from([Button::Left, Button::Right, Button::Middle]).contains([Button::Left]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right]).contains([Button::Left]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left]).contains([Button::Left]),
            true
        );
        assert_eq!(BitFlags::from([]).contains([Button::Left]), false);

        assert_eq!(
            BitFlags::from([Button::Left, Button::Right, Button::Middle])
                .contains([Button::Left, Button::Right]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right]).contains([Button::Left, Button::Right]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left]).contains([Button::Left, Button::Right]),
            false
        );
        assert_eq!(
            BitFlags::from([Button::Left]).contains([Button::Left, Button::Right, Button::Middle]),
            false
        );
    }

    #[test]
    fn test_intersects() {
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right, Button::Middle]).intersects([]),
            false
        );
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right]).intersects([]),
            false
        );
        assert_eq!(BitFlags::from([Button::Left]).intersects([]), false);
        assert_eq!(BitFlags::from([] as [Button; 0]).intersects([]), false);

        assert_eq!(
            BitFlags::from([Button::Left, Button::Right, Button::Middle])
                .intersects([Button::Left]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right]).intersects([Button::Left]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left]).intersects([Button::Left]),
            true
        );
        assert_eq!(BitFlags::from([]).intersects([Button::Left]), false);

        assert_eq!(
            BitFlags::from([Button::Left, Button::Right, Button::Middle])
                .intersects([Button::Left, Button::Right]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right]).intersects([Button::Left, Button::Right]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left]).intersects([Button::Left, Button::Right]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left]).intersects([
                Button::Left,
                Button::Right,
                Button::Middle
            ]),
            true
        );
    }

    #[test]
    fn test_iter() {
        assert_eq!(
            BitFlags::from([] as [Button; 0]).iter().collect::<Vec<_>>(),
            vec![]
        );

        assert_eq!(
            BitFlags::from([Button::Left]).iter().collect::<Vec<_>>(),
            vec![Button::Left]
        );
        assert_eq!(
            BitFlags::from([Button::Right]).iter().collect::<Vec<_>>(),
            vec![Button::Right]
        );
        assert_eq!(
            BitFlags::from([Button::Middle]).iter().collect::<Vec<_>>(),
            vec![Button::Middle]
        );

        assert_eq!(
            BitFlags::from([Button::Left, Button::Right])
                .iter()
                .collect::<Vec<_>>(),
            vec![Button::Left, Button::Right]
        );
        assert_eq!(
            BitFlags::from([Button::Left, Button::Middle])
                .iter()
                .collect::<Vec<_>>(),
            vec![Button::Left, Button::Middle]
        );
        assert_eq!(
            BitFlags::from([Button::Right, Button::Middle])
                .iter()
                .collect::<Vec<_>>(),
            vec![Button::Right, Button::Middle]
        );

        assert_eq!(
            BitFlags::from([Button::Left, Button::Right, Button::Middle])
                .iter()
                .collect::<Vec<_>>(),
            vec![Button::Left, Button::Right, Button::Middle]
        );
    }

    #[test]
    fn test_bit_or() {
        let buttons = BitFlags::new();
        assert_eq!(buttons | Button::Left, [Button::Left].into());
        assert_eq!(
            buttons | Button::Left | Button::Right,
            [Button::Left, Button::Right].into()
        );
    }

    #[test]
    fn test_bit_or_assign() {
        let mut buttons = BitFlags::new();
        buttons |= Button::Left;
        assert_eq!(buttons, [Button::Left].into());

        let mut buttons = BitFlags::new();
        buttons |= Button::Left;
        buttons |= Button::Right;
        assert_eq!(buttons, [Button::Left, Button::Right].into());
    }

    #[test]
    fn test_bit_and() {
        let buttons = BitFlags::from([Button::Left, Button::Right]);
        assert_eq!(buttons & Button::Left, [Button::Left].into());
        assert_eq!(buttons & Button::Right, [Button::Right].into());
        assert_eq!(buttons & Button::Middle, [].into());
    }

    #[test]
    fn test_bit_and_assign() {
        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons &= Button::Left;
        assert_eq!(buttons, [Button::Left].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons &= Button::Right;
        assert_eq!(buttons, [Button::Right].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons &= Button::Middle;
        assert_eq!(buttons, [Button::Middle].into());
    }

    #[test]
    fn test_bit_xor() {
        let buttons = BitFlags::from([Button::Left, Button::Right]);
        assert_eq!(buttons ^ Button::Left, [Button::Right].into());
        assert_eq!(buttons ^ Button::Right, [Button::Left].into());
        assert_eq!(
            buttons ^ Button::Middle,
            [Button::Left, Button::Right, Button::Middle].into()
        );
    }

    #[test]
    fn test_bit_xor_assign() {
        let mut buttons = BitFlags::from([Button::Left, Button::Right]);
        buttons ^= Button::Left;
        assert_eq!(buttons, [Button::Right].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right]);
        buttons ^= Button::Right;
        assert_eq!(buttons, [Button::Left].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right]);
        buttons ^= Button::Middle;
        assert_eq!(
            buttons,
            [Button::Left, Button::Right, Button::Middle].into()
        );
    }

    #[test]
    fn test_sub() {
        let buttons = BitFlags::from([Button::Left, Button::Right]);
        assert_eq!(buttons - Button::Left, [Button::Right].into());
        assert_eq!(buttons - Button::Right, [Button::Left].into());
        assert_eq!(
            buttons - Button::Middle,
            [Button::Left, Button::Right].into()
        );
    }

    #[test]
    fn test_sub_assign() {
        let mut buttons = BitFlags::from([Button::Left, Button::Right]);
        buttons -= Button::Left;
        assert_eq!(buttons, [Button::Right].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right]);
        buttons -= Button::Right;
        assert_eq!(buttons, [Button::Left].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right]);
        buttons -= Button::Middle;
        assert_eq!(buttons, [Button::Left, Button::Right].into());
    }
}
