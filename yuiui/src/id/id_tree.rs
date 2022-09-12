use std::collections::VecDeque;

use super::{Id, IdPathBuf};

#[derive(Debug, Clone)]
pub struct IdTree<T> {
    arena: Vec<Node<T>>,
}

impl<T> IdTree<T> {
    #[inline]
    pub fn root(&self) -> Cursor<'_, T> {
        Cursor::new(&self.arena)
    }

    #[inline]
    pub fn descendants(&self) -> Descendants<'_, T> {
        Descendants::new(&self.arena)
    }
}

impl<T> FromIterator<(IdPathBuf, T)> for IdTree<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (IdPathBuf, T)>,
    {
        let mut arena = Vec::new();
        let mut queue = VecDeque::new();
        let mut index = 0;

        {
            let mut last_head = Id::ROOT;
            let mut value = None;
            let mut children = Vec::new();

            for (id_path, child_value) in iter {
                if let Some(&head) = id_path.first() {
                    if head != last_head {
                        index += 1;
                        children.push(index);
                        last_head = head;
                    }
                    queue.push_back((head, 1, id_path, child_value));
                } else {
                    value = Some(child_value);
                }
            }

            arena.push(Node::new(Id::ROOT, value, children));
        }

        while let Some((head, tail, id_path, value)) = queue.pop_front() {
            let mut children = Vec::new();

            let value = if let Some(&tail_head) = id_path.get(tail) {
                index += 1;
                children.push(index);
                queue.push_back((tail_head, tail + 1, id_path, value));
                None
            } else {
                Some(value)
            };

            while let Some((next_head, _, _, _)) = queue.front() {
                if *next_head == head {
                    let (_, next_tail, id_path, value) = queue.pop_front().unwrap();
                    index += 1;
                    children.push(index);
                    if let Some(next_tail_head) = id_path.get(next_tail) {
                        queue.push_back((*next_tail_head, next_tail + 1, id_path, value));
                    }
                } else {
                    break;
                }
            }

            arena.push(Node::new(head, value, children));
        }

        Self { arena }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node<T> {
    id: Id,
    value: Option<T>,
    children: Vec<usize>,
}

impl<T> Node<T> {
    fn new(id: Id, value: Option<T>, children: Vec<usize>) -> Self {
        Self {
            id,
            value,
            children,
        }
    }

    #[inline]
    pub fn id(&self) -> Id {
        self.id
    }

    #[inline]
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

#[derive(Debug)]
pub struct Cursor<'a, T> {
    arena: &'a Vec<Node<T>>,
    node: &'a Node<T>,
}

impl<'a, T> Cursor<'a, T> {
    fn new(arena: &'a Vec<Node<T>>) -> Self {
        Self {
            arena,
            node: &arena[0],
        }
    }

    #[inline]
    pub fn current(&self) -> &Node<T> {
        &self.node
    }

    #[inline]
    pub fn children(&self) -> Children<'a, T> {
        Children::new(self.arena, &self.node.children)
    }
}

#[derive(Debug)]
pub struct Descendants<'a, T> {
    arena: &'a Vec<Node<T>>,
    current: usize,
}

impl<'a, T> Descendants<'a, T> {
    fn new(arena: &'a Vec<Node<T>>) -> Self {
        Self { arena, current: 0 }
    }
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = Cursor<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.arena.len() {
            let cursor = Cursor {
                arena: self.arena,
                node: &self.arena[self.current],
            };
            self.current += 1;
            Some(cursor)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Children<'a, T> {
    arena: &'a Vec<Node<T>>,
    children: &'a Vec<usize>,
    current: usize,
}

impl<'a, T> Children<'a, T> {
    fn new(arena: &'a Vec<Node<T>>, children: &'a Vec<usize>) -> Self {
        Self {
            arena,
            children,
            current: 0,
        }
    }
}

impl<'a, T> Iterator for Children<'a, T> {
    type Item = Cursor<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.children.len() {
            let current = self.children[self.current];
            let cursor = Cursor {
                arena: self.arena,
                node: &self.arena[current],
            };
            self.current += 1;
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
    fn test_id_tree() {
        //      +-- *1 --- *4
        //      |
        // *0 --+--- 2 -+- *5
        //      |       |
        //      |       +- *6
        //      |
        //      +--- 3 ---- 7 --- *8
        let id_tree = IdTree::from_iter([
            (vec![], 1),
            (vec![Id(1)], 2),
            (vec![Id(1), Id(4)], 3),
            (vec![Id(2), Id(5)], 4),
            (vec![Id(2), Id(6)], 5),
            (vec![Id(3), Id(7), Id(8)], 6),
        ]);

        assert_eq!(
            id_tree.root().current(),
            &Node::new(Id::ROOT, Some(1), vec![1, 2, 3])
        );

        let children = id_tree
            .root()
            .children()
            .map(|cursor| cursor.current().clone())
            .collect::<Vec<_>>();
        assert_eq!(
            children,
            vec![
                Node::new(Id(1), Some(2), vec![4]),
                Node::new(Id(2), None, vec![5, 6]),
                Node::new(Id(3), None, vec![7]),
            ]
        );

        let descendants = id_tree
            .descendants()
            .map(|cursor| cursor.current().clone())
            .collect::<Vec<_>>();
        assert_eq!(
            descendants,
            vec![
                Node::new(Id(0), Some(1), vec![1, 2, 3]),
                Node::new(Id(1), Some(2), vec![4]),
                Node::new(Id(2), None, vec![5, 6]),
                Node::new(Id(3), None, vec![7]),
                Node::new(Id(4), Some(3), vec![]),
                Node::new(Id(5), Some(4), vec![]),
                Node::new(Id(6), Some(5), vec![]),
                Node::new(Id(7), None, vec![8]),
                Node::new(Id(8), Some(6), vec![]),
            ]
        );
    }
}
