use super::{Depth, IdPath, IdPathBuf};

#[derive(Debug, Clone)]
pub struct IdStack {
    id_path: IdPathBuf,
    stack: Vec<(usize, Depth)>,
}

impl IdStack {
    #[inline]
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            stack: Vec::new(),
        }
    }

    #[inline]
    pub unsafe fn from_external(id_path: IdPathBuf, stack: Vec<(usize, Depth)>) -> Self {
        if cfg!(debug_assertions) {
            ensure_valid(&id_path, &stack);
        }
        Self { id_path, stack }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<(&IdPath, Depth)> {
        if let Some((len, depth)) = self.stack.get(index) {
            Some((&self.id_path[..*len], *depth))
        } else {
            None
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<(&IdPath, Depth)> {
        if let Some((len, depth)) = self.stack.last() {
            Some((&self.id_path[..*len], *depth))
        } else {
            None
        }
    }

    #[inline]
    pub fn push(&mut self, id_path: &IdPath, depth: Depth) {
        assert!(id_path.starts_with(&self.id_path));
        self.stack.push((id_path.len(), depth));
        self.id_path
            .extend_from_slice(&id_path[self.id_path.len()..]);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<(&IdPath, Depth)> {
        if let Some((len, depth)) = self.stack.pop() {
            self.id_path.truncate(len);
            Some((&self.id_path, depth))
        } else {
            None
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    id_stack: &'a IdStack,
    index: usize,
}

impl<'a> Iter<'a> {
    fn new(id_stack: &'a IdStack) -> Self {
        Self { id_stack, index: 0 }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a IdPath, Depth);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((id_path, depth)) = self.id_stack.get(self.index) {
            self.index += 1;
            Some((id_path, depth))
        } else {
            None
        }
    }
}

fn ensure_valid(id_path: &IdPath, stack: &[(usize, Depth)]) {
    let mut last_len = 0;
    for (len, _) in stack {
        assert!(*len <= id_path.len());
        assert!(*len >= last_len);
        last_len = *len;
    }
}

#[cfg(test)]
mod tests {
    use super::super::Id;
    use super::*;

    #[test]
    fn test_id_stack() {
        let mut id_stack = IdStack::new();

        id_stack.push(&[], 1);
        id_stack.push(&[Id(0)], 0);
        id_stack.push(&[Id(0), Id(1)], 0);
        id_stack.push(&[Id(0), Id(1)], 1);
        id_stack.push(&[Id(0), Id(1), Id(2)], 1);

        assert_eq!(id_stack.len(), 5);

        assert_eq!(id_stack.get(0), Some((&[] as &IdPath, 1)));
        assert_eq!(id_stack.get(1), Some((&[Id(0)] as &IdPath, 0)));
        assert_eq!(id_stack.get(2), Some((&[Id(0), Id(1)] as &IdPath, 0)));
        assert_eq!(id_stack.get(3), Some((&[Id(0), Id(1)] as &IdPath, 1)));
        assert_eq!(
            id_stack.get(4),
            Some((&[Id(0), Id(1), Id(2)] as &IdPath, 1))
        );
        assert_eq!(id_stack.get(5), None);

        let mut iter = id_stack.iter();
        assert_eq!(iter.next(), Some((&[] as &IdPath, 1)));
        assert_eq!(iter.next(), Some((&[Id(0)] as &IdPath, 0)));
        assert_eq!(iter.next(), Some((&[Id(0), Id(1)] as &IdPath, 0)));
        assert_eq!(iter.next(), Some((&[Id(0), Id(1)] as &IdPath, 1)));
        assert_eq!(iter.next(), Some((&[Id(0), Id(1), Id(2)] as &IdPath, 1)));
        assert_eq!(iter.next(), None);

        assert_eq!(
            id_stack.peek(),
            Some((&[Id(0), Id(1), Id(2)] as &IdPath, 1))
        );
        assert_eq!(id_stack.pop(), Some((&[Id(0), Id(1), Id(2)] as &IdPath, 1)));

        assert_eq!(id_stack.peek(), Some((&[Id(0), Id(1)] as &IdPath, 1)));
        assert_eq!(id_stack.pop(), Some((&[Id(0), Id(1)] as &IdPath, 1)));

        assert_eq!(id_stack.peek(), Some((&[Id(0), Id(1)] as &IdPath, 0)));
        assert_eq!(id_stack.pop(), Some((&[Id(0), Id(1)] as &IdPath, 0)));

        assert_eq!(id_stack.peek(), Some((&[Id(0)] as &IdPath, 0)));
        assert_eq!(id_stack.pop(), Some((&[Id(0)] as &IdPath, 0)));

        assert_eq!(id_stack.peek(), Some((&[] as &IdPath, 1)));
        assert_eq!(id_stack.pop(), Some((&[] as &IdPath, 1)));

        assert_eq!(id_stack.peek(), None);
        assert_eq!(id_stack.pop(), None);

        assert_eq!(id_stack.len(), 0);
    }
}
