#![allow(dead_code)]

use std::array;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::BitXor;
use std::ops::BitXorAssign;

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
                .fold(0 as usize, |flags, value| flags | value.into() as usize),
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
                .fold(0 as usize, |flags, value| flags | value.into() as usize),
            _type: PhantomData,
        }
    }
}

impl<T: Into<usize>> BitAnd<T> for BitFlags<T> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: T) -> Self::Output {
        Self {
            flags: self.flags & rhs.into(),
            _type: self._type,
        }
    }
}

impl<T: Into<usize>> BitAndAssign<T> for BitFlags<T> {
    #[inline]
    fn bitand_assign(&mut self, rhs: T) {
        self.flags &= rhs.into();
    }
}

impl<T: Into<usize>> BitOr<T> for BitFlags<T> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: T) -> Self::Output {
        Self {
            flags: self.flags | rhs.into(),
            _type: self._type,
        }
    }
}

impl<T: Into<usize>> BitOrAssign<T> for BitFlags<T> {
    #[inline]
    fn bitor_assign(&mut self, rhs: T) {
        self.flags |= rhs.into();
    }
}

impl<T: Into<usize>> BitXor<T> for BitFlags<T> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: T) -> Self::Output {
        Self {
            flags: self.flags ^ rhs.into(),
            _type: self._type,
        }
    }
}

impl<T: Into<usize>> BitXorAssign<T> for BitFlags<T> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: T) {
        self.flags ^= rhs.into();
    }
}

impl<T> BitAnd<BitFlags<T>> for BitFlags<T> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: BitFlags<T>) -> Self::Output {
        Self {
            flags: self.flags & rhs.flags,
            _type: self._type,
        }
    }
}

impl<T> BitAndAssign<BitFlags<T>> for BitFlags<T> {
    #[inline]
    fn bitand_assign(&mut self, rhs: BitFlags<T>) {
        self.flags &= rhs.flags;
    }
}

impl<T> BitOr<BitFlags<T>> for BitFlags<T> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: BitFlags<T>) -> Self::Output {
        Self {
            flags: self.flags | rhs.flags,
            _type: self._type,
        }
    }
}

impl<T> BitOrAssign<BitFlags<T>> for BitFlags<T> {
    #[inline]
    fn bitor_assign(&mut self, rhs: BitFlags<T>) {
        self.flags |= rhs.flags;
    }
}

impl<T> BitXor<BitFlags<T>> for BitFlags<T> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: BitFlags<T>) -> Self::Output {
        Self {
            flags: self.flags ^ rhs.flags,
            _type: self._type,
        }
    }
}

impl<T> BitXorAssign<BitFlags<T>> for BitFlags<T> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: BitFlags<T>) {
        self.flags ^= rhs.flags;
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
        let mut buttons = BitFlags::new();
        buttons = buttons | Button::Left;
        assert_eq!(buttons, [Button::Left].into());

        let mut buttons = BitFlags::new();
        buttons = buttons | BitFlags::from(Button::Left);
        assert_eq!(buttons, [Button::Left].into());
    }

    #[test]
    fn test_bit_or_assign() {
        let mut buttons = BitFlags::new();
        buttons |= Button::Left;
        assert_eq!(buttons, [Button::Left].into());

        let mut buttons = BitFlags::new();
        buttons |= BitFlags::from(Button::Left);
        assert_eq!(buttons, [Button::Left].into());
    }

    #[test]
    fn test_bit_and() {
        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons = buttons & Button::Left;
        assert_eq!(buttons, [Button::Left].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons = buttons & BitFlags::from(Button::Left);
        assert_eq!(buttons, [Button::Left].into());
    }

    #[test]
    fn test_bit_and_assign() {
        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons &= Button::Left;
        assert_eq!(buttons, [Button::Left].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons &= BitFlags::from(Button::Left);
        assert_eq!(buttons, [Button::Left].into());
    }

    #[test]
    fn test_bit_xor() {
        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons = buttons ^ Button::Left;
        assert_eq!(buttons, [Button::Right, Button::Middle].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons = buttons ^ BitFlags::from(Button::Left);
        assert_eq!(buttons, [Button::Right, Button::Middle].into());
    }

    #[test]
    fn test_bit_xor_assign() {
        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons ^= Button::Left;
        assert_eq!(buttons, [Button::Right, Button::Middle].into());

        let mut buttons = BitFlags::from([Button::Left, Button::Right, Button::Middle]);
        buttons ^= BitFlags::from(Button::Left);
        assert_eq!(buttons, [Button::Right, Button::Middle].into());
    }
}
