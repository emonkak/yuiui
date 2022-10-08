use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::mem;

use crate::bits::Bits;

#[derive(Debug)]
pub struct Iter<T, B> {
    bits: B,
    _phantom: PhantomData<T>,
}

impl<T, B> Iter<T, B> {
    pub(super) fn new(bits: B) -> Self {
        assert_eq!(mem::size_of::<B>(), mem::size_of::<T>());
        assert_eq!(mem::align_of::<B>(), mem::align_of::<T>());
        Self {
            bits,
            _phantom: PhantomData,
        }
    }
}

impl<T, B: Bits> Iterator for Iter<T, B> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits != B::ZERO {
            let trailing = self.bits.trailing_zeros();
            let flag = B::ONE << trailing;
            let value = unsafe { mem::transmute_copy(&flag) };
            self.bits &= self.bits.wrapping_sub(B::ONE);
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.bits.count_ones() as usize;
        (count, Some(count))
    }

    fn count(self) -> usize {
        self.bits.count_ones() as usize
    }
}

impl<T, B: Bits> FusedIterator for Iter<T, B> {}
