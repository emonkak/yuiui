use std::collections::VecDeque;

use super::{Id, IdPath, IdPathBuf};

#[derive(Debug, Clone)]
pub struct IdTree<T> {
    arena: Vec<Node<T>>,
    len: usize,
}

impl<T> IdTree<T> {
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut arena = Vec::with_capacity(capacity + 1);
        let root = Node::new(Id::ROOT, None, Vec::new());
        arena.push(root);
        Self { arena, len: 0 }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn root(&self) -> Cursor<'_, T> {
        Cursor::new(&self.arena, 0)
    }

    pub fn insert(&mut self, id_path: &IdPath, data: T) {
        let key = self.insertion_point(id_path);
        let node = &mut self.arena[key];
        if node.data.is_none() {
            self.len += 1;
        }
        node.data = Some(data);
    }

    pub fn insert_or_update(&mut self, id_path: &IdPath, data: T, f: impl FnOnce(T, T) -> T) {
        let key = self.insertion_point(id_path);
        let node = &mut self.arena[key];
        let data = if let Some(old_data) = node.data.take() {
            f(old_data, data)
        } else {
            self.len += 1;
            data
        };
        node.data = Some(data);
    }

    fn insertion_point(&mut self, mut id_path: &IdPath) -> usize {
        let mut key = 0;

        'outer: while let Some((&head, tail)) = id_path.split_first() {
            for &child in &self.arena[key].children {
                if self.arena[child].id == head {
                    key = child;
                    id_path = tail;
                    continue 'outer;
                }
            }

            let next_key = self.arena.len();
            self.arena[key].children.push(next_key);

            let node = Node::new(head, None, Vec::new());
            self.arena.push(node);

            key = next_key;
            id_path = tail;
        }

        key
    }
}

impl<T> Default for IdTree<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FromIterator<&'a IdPathBuf> for IdTree<()> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a IdPathBuf>,
    {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut id_tree = Self::with_capacity(lower);
        for id_path in iter {
            id_tree.insert(id_path, ());
        }
        id_tree
    }
}

impl<T> FromIterator<(IdPathBuf, T)> for IdTree<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (IdPathBuf, T)>,
    {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut id_tree = Self::with_capacity(lower);
        for (id_path, value) in iter {
            id_tree.insert(&id_path, value);
        }
        id_tree
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node<T> {
    id: Id,
    data: Option<T>,
    children: Vec<usize>,
}

impl<T> Node<T> {
    fn new(id: Id, data: Option<T>, children: Vec<usize>) -> Self {
        Self { id, data, children }
    }

    #[inline]
    pub fn id(&self) -> Id {
        self.id
    }

    #[inline]
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }
}

#[derive(Debug)]
pub struct Cursor<'a, T> {
    arena: &'a [Node<T>],
    node: &'a Node<T>,
}

impl<'a, T> Cursor<'a, T> {
    fn new(arena: &'a [Node<T>], key: usize) -> Self {
        Self {
            arena,
            node: &arena[key],
        }
    }

    #[inline]
    pub fn current(&self) -> &Node<T> {
        &self.node
    }

    #[inline]
    pub fn children(&self) -> Children<'a, T> {
        Children::new(&self.node.children, self.arena)
    }

    #[inline]
    pub fn descendants(&self) -> Descendants<'a, T> {
        Descendants::new(&self.node.children, self.arena)
    }
}

#[derive(Debug)]
pub struct Children<'a, T> {
    children: &'a [usize],
    arena: &'a [Node<T>],
    current: usize,
}

impl<'a, T> Children<'a, T> {
    fn new(children: &'a [usize], arena: &'a [Node<T>]) -> Self {
        Self {
            children,
            arena,
            current: 0,
        }
    }
}

impl<'a, T> Iterator for Children<'a, T> {
    type Item = Cursor<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.children.len() {
            let child = self.children[self.current];
            let cursor = Cursor::new(self.arena, child);
            self.current += 1;
            Some(cursor)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Descendants<'a, T> {
    queue: VecDeque<usize>,
    arena: &'a [Node<T>],
}

impl<'a, T> Descendants<'a, T> {
    fn new(children: &'a [usize], arena: &'a [Node<T>]) -> Self {
        let queue = VecDeque::from_iter(children.into_iter().copied());
        Self { queue, arena }
    }
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = Cursor<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(child) = self.queue.pop_front() {
            let cursor = Cursor::new(self.arena, child);
            self.queue.extend(cursor.node.children.iter().copied());
            Some(cursor)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        //      +-- *2 --- *5
        //      |
        // *1 --+--- 3 -+- *6
        //      |       |
        //      |       +- *7
        //      |
        //      +--- 4 ---- 8 --- *9
        let id_tree = IdTree::from_iter([
            (vec![], 1),
            (vec![Id::new(2)], 2),
            (vec![Id::new(2), Id::new(5)], 3),
            (vec![Id::new(3), Id::new(6)], 4),
            (vec![Id::new(3), Id::new(7)], 5),
            (vec![Id::new(4), Id::new(8), Id::new(9)], 6),
        ]);

        let cursor = id_tree.root();
        assert_eq!(cursor.current().id(), Id::new(1));
        assert_eq!(cursor.current().data().copied(), Some(1));

        let children = id_tree
            .root()
            .children()
            .map(|cursor| (cursor.current().id(), cursor.current().data().copied()))
            .collect::<Vec<_>>();
        assert_eq!(
            children,
            vec![
                (Id::new(2), Some(2)),
                (Id::new(3), None),
                (Id::new(4), None),
            ]
        );

        let descendants = id_tree
            .root()
            .descendants()
            .map(|cursor| (cursor.current().id(), cursor.current().data().copied()))
            .collect::<Vec<_>>();
        assert_eq!(
            descendants,
            vec![
                (Id::new(2), Some(2)),
                (Id::new(3), None),
                (Id::new(4), None),
                (Id::new(5), Some(3)),
                (Id::new(6), Some(4)),
                (Id::new(7), Some(5)),
                (Id::new(8), None),
                (Id::new(9), Some(6)),
            ]
        );
    }
}
