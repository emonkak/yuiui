use std::iter::FusedIterator;

#[derive(Debug)]
pub struct BitIter(usize);

impl From<usize> for BitIter {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Iterator for BitIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            let trailing = self.0.trailing_zeros() as usize;
            self.0 &= self.0.wrapping_sub(1);
            Some(trailing)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.0.count_ones() as usize;
        (count, Some(count))
    }

    fn count(self) -> usize {
        self.0.count_ones() as usize
    }
}

impl FusedIterator for BitIter {}
