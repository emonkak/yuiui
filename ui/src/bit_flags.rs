#![allow(dead_code)]

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

impl<T: Into<usize>> BitFlags<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            flags: 0,
            _type: PhantomData,
        }
    }

    #[inline]
    pub fn from_flags(flags: usize) -> Self {
        Self {
            flags,
            _type: PhantomData,
        }
    }

    #[inline]
    pub fn flags(&self) -> usize {
        self.flags
    }

    #[inline]
    pub fn contains(&self, value: T) -> bool {
        (self.flags & value.into()) != 0
    }

    #[inline]
    pub fn intersects(&self, other: Self) -> bool {
        (self.flags & other.flags) == other.flags
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

impl<T: Into<usize>> FromIterator<T> for BitFlags<T> {
    fn from_iter<I: IntoIterator<Item=T>>(values: I) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_ops() {
        let mut buttons = BitFlags::from_iter([Button::Left, Button::Right]);

        assert_eq!(buttons.contains(Button::None), false);
        assert_eq!(buttons.contains(Button::Left), true);
        assert_eq!(buttons.contains(Button::Right), true);
        assert_eq!(buttons.contains(Button::Middle), false);

        buttons |= Button::Middle;

        assert_eq!(buttons.contains(Button::None), false);
        assert_eq!(buttons.contains(Button::Left), true);
        assert_eq!(buttons.contains(Button::Right), true);
        assert_eq!(buttons.contains(Button::Middle), true);

        buttons &= Button::Left;

        assert_eq!(buttons.contains(Button::None), false);
        assert_eq!(buttons.contains(Button::Left), true);
        assert_eq!(buttons.contains(Button::Right), false);
        assert_eq!(buttons.contains(Button::Middle), false);

        buttons ^= Button::Left;

        assert_eq!(buttons.contains(Button::None), false);
        assert_eq!(buttons.contains(Button::Left), false);
        assert_eq!(buttons.contains(Button::Right), false);
        assert_eq!(buttons.contains(Button::Middle), false);
    }

    #[test]
    fn test_intersects() {
        assert_eq!(
            BitFlags::from_iter([Button::Left, Button::Right, Button::Middle])
                .intersects(BitFlags::new()),
            true
        );
        assert_eq!(
            BitFlags::from_iter([Button::Left, Button::Right])
                .intersects(BitFlags::new()),
            true
        );
        assert_eq!(
            BitFlags::from_iter([Button::Left])
                .intersects(BitFlags::new()),
            true
        );
        assert_eq!(
            BitFlags::from_iter([] as [Button; 0])
                .intersects(BitFlags::new()),
            true
        );

        assert_eq!(
            BitFlags::from_iter([Button::Left, Button::Right, Button::Middle])
                .intersects(BitFlags::from_iter([Button::Left])),
            true
        );
        assert_eq!(
            BitFlags::from_iter([Button::Left, Button::Right])
                .intersects(BitFlags::from_iter([Button::Left])),
            true
        );
        assert_eq!(
            BitFlags::from_iter([Button::Left])
                .intersects(BitFlags::from_iter([Button::Left])),
            true
        );
        assert_eq!(
            BitFlags::from_iter([])
                .intersects(BitFlags::from_iter([Button::Left])),
            false
        );

        assert_eq!(
            BitFlags::from_iter([Button::Left, Button::Right, Button::Middle])
                .intersects(BitFlags::from_iter([Button::Left, Button::Right])),
            true
        );
        assert_eq!(
            BitFlags::from_iter([Button::Left, Button::Right])
                .intersects(BitFlags::from_iter([Button::Left, Button::Right])),
            true
        );
        assert_eq!(
            BitFlags::from_iter([Button::Left])
                .intersects(BitFlags::from_iter([Button::Left, Button::Right])),
            false
        );
    }
}
