#![allow(dead_code)]

use std::array;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitFlags<T> {
    flags: usize,
    _type: PhantomData<T>,
}

impl<T> BitFlags<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            flags: 0,
            _type: PhantomData,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.flags == 0
    }

    #[inline]
    pub fn contains<U: Into<Self>>(&self, other: U) -> bool {
        let flags = other.into().flags;
        (self.flags & flags) != 0
    }

    #[inline]
    pub fn intersects<U: Into<Self>>(&self, other: U) -> bool {
        let flags = other.into().flags;
        (self.flags & flags) == flags
    }
}

impl<T: Into<usize>> From<T> for BitFlags<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self {
            flags: value.into(),
            _type: PhantomData,
        }
    }
}

impl<T: Into<usize>, const N: usize> From<[T; N]> for BitFlags<T> {
    #[inline]
    fn from(values: [T; N]) -> Self {
        Self {
            flags: array::IntoIter::new(values)
                .fold(0 as usize, |flags, value| flags | value.into()),
            _type: PhantomData,
        }
    }
}

impl<T: Into<usize>> FromIterator<T> for BitFlags<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        Self {
            flags: values
                .into_iter()
                .fold(0 as usize, |flags, value| flags | value.into()),
            _type: PhantomData,
        }
    }
}

impl<T: Into<BitFlags<U>>, U> BitAnd<T> for BitFlags<U> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: T) -> Self::Output {
        Self {
            flags: self.flags & rhs.into().flags,
            _type: self._type,
        }
    }
}

impl<T: Into<BitFlags<U>>, U> BitAndAssign<T> for BitFlags<U> {
    #[inline]
    fn bitand_assign(&mut self, rhs: T) {
        self.flags &= rhs.into().flags;
    }
}

impl<T: Into<BitFlags<U>>, U> BitOr<T> for BitFlags<U> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: T) -> Self::Output {
        Self {
            flags: self.flags | rhs.into().flags,
            _type: self._type,
        }
    }
}

impl<T: Into<BitFlags<U>>, U> BitOrAssign<T> for BitFlags<U> {
    #[inline]
    fn bitor_assign(&mut self, rhs: T) {
        self.flags |= rhs.into().flags;
    }
}

impl<T: Into<BitFlags<U>>, U> BitXor<T> for BitFlags<U> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: T) -> Self::Output {
        Self {
            flags: self.flags ^ rhs.into().flags,
            _type: self._type,
        }
    }
}

impl<T: Into<BitFlags<U>>, U> BitXorAssign<T> for BitFlags<U> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: T) {
        self.flags ^= rhs.into().flags;
    }
}

impl<T: Into<BitFlags<U>>, U> Sub<T> for BitFlags<U> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: T) -> Self::Output {
        Self {
            flags: self.flags & !rhs.into().flags,
            _type: self._type,
        }
    }
}

impl<T: Into<BitFlags<U>>, U> SubAssign<T> for BitFlags<U> {
    #[inline]
    fn sub_assign(&mut self, rhs: T) {
        self.flags &= !rhs.into().flags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Button {
        None = 0b000,
        Left = 0b001,
        Right = 0b010,
        Middle = 0b100,
    }

    impl Into<usize> for Button {
        fn into(self) -> usize {
            self as usize
        }
    }

    #[test]
    fn test_is_empty() {
        assert_eq!((BitFlags::new() as BitFlags<Button>).is_empty(), true);
        assert_eq!((BitFlags::from([]) as BitFlags<Button>).is_empty(), true);
        assert_eq!((BitFlags::from(Button::None)).is_empty(), true);
        assert_eq!((BitFlags::from(Button::Left)).is_empty(), false);
        assert_eq!((BitFlags::from(Button::Right)).is_empty(), false);
        assert_eq!((BitFlags::from(Button::Middle)).is_empty(), false);
    }

    #[test]
    fn test_contains() {
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right, Button::Middle]).contains([]),
            false
        );
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right]).contains([]),
            false
        );
        assert_eq!(BitFlags::from([Button::Left]).contains([]), false);
        assert_eq!(BitFlags::from([] as [Button; 0]).contains([]), false);

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
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left]).contains([Button::Left, Button::Right, Button::Middle]),
            true
        );
    }

    #[test]
    fn test_intersects() {
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right, Button::Middle]).intersects([]),
            true
        );
        assert_eq!(
            BitFlags::from([Button::Left, Button::Right]).intersects([]),
            true
        );
        assert_eq!(BitFlags::from([Button::Left]).intersects([]), true);
        assert_eq!(BitFlags::from([] as [Button; 0]).intersects([]), true);

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
            false
        );
        assert_eq!(
            BitFlags::from([Button::Left]).intersects([
                Button::Left,
                Button::Right,
                Button::Middle
            ]),
            false
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
