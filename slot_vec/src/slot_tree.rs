use crate::slot_vec::{Key, SlotVec};

use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(NonZeroUsize);

impl NodeId {
    pub const ROOT: Self = Self(unsafe { NonZeroUsize::new_unchecked(1) });

    fn new(key: Key) -> Self {
        assert!(key > 0);
        Self(unsafe { NonZeroUsize::new_unchecked(key) })
    }
}

impl Into<Key> for NodeId {
    fn into(self) -> Key {
        self.0.get()
    }
}

#[derive(Debug)]
pub struct SlotTree<T> {
    arena: SlotVec<Node<T>>,
}

impl<T> SlotTree<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        let mut arena = SlotVec::new();
        let root = Node::new(data, None, Vec::new());
        arena.reserve();
        arena.push(root);
        Self { arena }
    }

    #[inline]
    pub fn contains(&self, node_id: NodeId) -> bool {
        self.arena.contains(node_id.into())
    }

    #[inline]
    pub fn next_id(&self) -> NodeId {
        NodeId::new(self.arena.next_key())
    }

    #[inline]
    pub fn get(&self, node_id: NodeId) -> Option<&Node<T>> {
        self.arena.get(node_id.into())
    }

    #[inline]
    pub fn get_mut(&mut self, node_id: NodeId) -> Option<&mut Node<T>> {
        self.arena.get_mut(node_id.into())
    }

    pub fn append(&mut self, parent: NodeId, data: T) -> NodeId {
        let child_node = Node::new(data, Some(parent), Vec::new());
        let child = NodeId::new(self.arena.push(child_node));
        let parent_node = &mut self.arena[parent.into()];
        parent_node.children.push(child);
        child
    }

    #[inline]
    pub fn iter_from(&self, origin: NodeId) -> IterFrom<'_, T> {
        IterFrom::new(&self.arena, origin)
    }

    #[inline]
    pub fn detach_from(&mut self, origin: NodeId) -> DetachFrom<'_, T> {
        DetachFrom::new(&mut self.arena, origin)
    }
}

impl<T> Index<NodeId> for SlotTree<T> {
    type Output = Node<T>;

    #[inline]
    fn index(&self, node_id: NodeId) -> &Self::Output {
        &self.arena[node_id.into()]
    }
}

impl<T> IndexMut<NodeId> for SlotTree<T> {
    #[inline]
    fn index_mut(&mut self, node_id: NodeId) -> &mut Self::Output {
        &mut self.arena[node_id.into()]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node<T> {
    data: T,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}

impl<T> Node<T> {
    fn new(data: T, parent: Option<NodeId>, children: Vec<NodeId>) -> Self {
        Self {
            data,
            parent,
            children,
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn into_data(self) -> T {
        self.data
    }

    pub fn children(&self) -> &[NodeId] {
        &self.children
    }
}

pub struct IterFrom<'a, T> {
    arena: &'a SlotVec<Node<T>>,
    queue: VecDeque<NodeId>,
}

impl<'a, T> IterFrom<'a, T> {
    fn new(arena: &'a SlotVec<Node<T>>, origin: NodeId) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(origin);
        Self { arena, queue }
    }
}

impl<'a, T> Iterator for IterFrom<'a, T> {
    type Item = (NodeId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node_id) = self.queue.pop_front() {
            let node = &self.arena[node_id.into()];
            self.queue.extend(&node.children);
            Some((node_id, &node.data))
        } else {
            None
        }
    }
}

pub struct DetachFrom<'a, T> {
    arena: &'a mut SlotVec<Node<T>>,
    queue: VecDeque<NodeId>,
    origin: NodeId,
}

impl<'a, T> DetachFrom<'a, T> {
    fn new(arena: &'a mut SlotVec<Node<T>>, origin: NodeId) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(origin);
        Self {
            arena,
            queue,
            origin,
        }
    }
}

impl<'a, T> Iterator for DetachFrom<'a, T> {
    type Item = (NodeId, T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.queue.pop_front() {
            let node = self.arena.remove(current.into()).unwrap();
            if current == self.origin {
                if let Some(parent) = node.parent {
                    let parent_node = &mut self.arena[parent.into()];
                    if let Some(position) = parent_node
                        .children
                        .iter()
                        .position(|child| *child == current)
                    {
                        parent_node.children.swap_remove(position);
                    }
                }
            }
            self.queue.extend(node.children);
            Some((current, node.data))
        } else {
            None
        }
    }
}

impl<'a, T> Drop for DetachFrom<'a, T> {
    fn drop(&mut self) {
        while let Some(_) = self.next() {}
    }
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use super::*;

    #[test]
    fn test_append() {
        let mut tree = SlotTree::new("foo");
        let foo = NodeId::ROOT;
        let bar = tree.append(foo, "bar");
        let baz = tree.append(bar, "baz");
        let qux = tree.append(foo, "qux");
        let quux = tree.next_id();

        assert_eq!(tree.contains(foo), true);
        assert_eq!(tree.contains(bar), true);
        assert_eq!(tree.contains(baz), true);
        assert_eq!(tree.contains(qux), true);
        assert_eq!(tree.contains(quux), false);

        assert_eq!(tree.get(foo), Some(&Node::new("foo", None, vec![bar, qux])));
        assert_eq!(tree.get(bar), Some(&Node::new("bar", Some(foo), vec![baz])));
        assert_eq!(tree.get(baz), Some(&Node::new("baz", Some(bar), vec![])));
        assert_eq!(tree.get(qux), Some(&Node::new("qux", Some(foo), vec![])));
        assert_eq!(tree.get(quux), None);

        assert_eq!(
            tree.get_mut(foo),
            Some(&mut Node::new("foo", None, vec![bar, qux]))
        );
        assert_eq!(
            tree.get_mut(bar),
            Some(&mut Node::new("bar", Some(foo), vec![baz]))
        );
        assert_eq!(
            tree.get_mut(baz),
            Some(&mut Node::new("baz", Some(bar), vec![]))
        );
        assert_eq!(
            tree.get_mut(qux),
            Some(&mut Node::new("qux", Some(foo), vec![]))
        );
        assert_eq!(tree.get_mut(quux), None);

        assert_eq!(&tree[foo], &Node::new("foo", None, vec![bar, qux]));
        assert_eq!(&tree[bar], &Node::new("bar", Some(foo), vec![baz]));
        assert_eq!(&tree[baz], &Node::new("baz", Some(bar), vec![]));
        assert_eq!(&tree[qux], &Node::new("qux", Some(foo), vec![]));
        assert!(catch_unwind(|| &tree[quux]).is_err());

        assert_eq!(&mut tree[foo], &mut Node::new("foo", None, vec![bar, qux]));
        assert_eq!(&mut tree[bar], &mut Node::new("bar", Some(foo), vec![baz]));
        assert_eq!(&mut tree[baz], &mut Node::new("baz", Some(bar), vec![]));
        assert_eq!(&mut tree[qux], &mut Node::new("qux", Some(foo), vec![]));
        assert!(catch_unwind(move || {
            let _ = &mut tree[quux];
        })
        .is_err());
    }

    #[test]
    fn test_iter_from() {
        let mut tree = SlotTree::new("foo");
        let foo = NodeId::ROOT;
        let bar = tree.append(foo, "bar");
        let baz = tree.append(bar, "baz");
        let qux = tree.append(foo, "qux");

        let xs: Vec<(NodeId, &str)> = tree
            .iter_from(foo)
            .map(|(key, value)| (key, *value))
            .collect();
        assert_eq!(
            xs,
            vec![(foo, "foo"), (bar, "bar"), (qux, "qux"), (baz, "baz")]
        );

        let xs: Vec<(NodeId, &str)> = tree
            .iter_from(bar)
            .map(|(key, value)| (key, *value))
            .collect();
        assert_eq!(xs, vec![(bar, "bar"), (baz, "baz")]);

        let xs: Vec<(NodeId, &str)> = tree
            .iter_from(baz)
            .map(|(key, value)| (key, *value))
            .collect();
        assert_eq!(xs, vec![(baz, "baz")]);

        let xs: Vec<(NodeId, &str)> = tree
            .iter_from(qux)
            .map(|(key, value)| (key, *value))
            .collect();
        assert_eq!(xs, vec![(qux, "qux")]);
    }

    #[test]
    fn test_detach_from() {
        let mut tree = SlotTree::new("foo");
        let foo = NodeId::ROOT;
        let bar = tree.append(foo, "bar");
        let baz = tree.append(bar, "baz");
        let qux = tree.append(foo, "qux");

        let xs: Vec<(NodeId, &str)> = tree.detach_from(bar).collect();
        assert_eq!(xs, vec![(bar, "bar"), (baz, "baz")]);

        assert_eq!(tree.contains(foo), true);
        assert_eq!(tree.contains(bar), false);
        assert_eq!(tree.contains(baz), false);
        assert_eq!(tree.contains(qux), true);

        let xs: Vec<(NodeId, &str)> = tree.detach_from(foo).collect();
        assert_eq!(xs, vec![(foo, "foo"), (qux, "qux")]);

        assert_eq!(tree.contains(foo), false);
        assert_eq!(tree.contains(bar), false);
        assert_eq!(tree.contains(baz), false);
        assert_eq!(tree.contains(qux), false);
    }
}
