use std::iter::FusedIterator;

use super::Id;

#[derive(Debug)]
pub struct IdCounter {
    count: usize,
}

impl IdCounter {
    pub fn new() -> Self {
        Self { count: 1 }
    }

    pub fn next(&mut self) -> Id {
        let id = self.count;
        self.count += 1;
        Id::new(id)
    }

    pub fn take(&mut self, n: usize) -> Take<'_> {
        Take::new(self, n)
    }
}

pub struct Take<'a> {
    counter: &'a mut IdCounter,
    n: usize,
}

impl<'a> Take<'a> {
    fn new(counter: &'a mut IdCounter, n: usize) -> Self {
        Self { counter, n }
    }
}

impl<'a> Iterator for Take<'a> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n == 0 {
            None
        } else {
            self.n -= 1;
            Some(self.counter.next())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.n, Some(self.n))
    }
}

impl<'a> FusedIterator for Take<'a> {}
