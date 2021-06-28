use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use slot_vec::SlotVec;

#[derive(Debug)]
pub struct Tree<T> {
    arena: SlotVec<Node<T>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Node<T> {
    data: T,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    parent: Option<NodeId>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DetachedNode<T> {
    data: T,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
}

pub type NodeId = usize;

impl<T> Tree<T> {
    pub fn new() -> Tree<T> {
        Tree {
            arena: SlotVec::new()
        }
    }

    pub fn is_attached(&self, target_id: NodeId) -> bool {
        self.arena.has(target_id)
    }

    pub fn attach(&mut self, node: impl Into<DetachedNode<T>>) -> NodeId {
        let node = Node::new(node.into(), None, None, None);
        self.register(node)
    }

    pub fn detach(&mut self, target_id: NodeId) -> DetachedNode<T> {
        let node = self.arena.remove(target_id);
        if !node.is_root() {
            // The root node cannot be detach.
            detach_node(&mut self.arena, &node);
        }
        DetachedNode {
            data: node.data,
            first_child: node.first_child,
            last_child: node.last_child,
        }
    }

    pub fn detach_subtree(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, Node<T>)> + '_ {
        DetachSubtree {
            root_id: target_id,
            next: Some(grandest_child(&self.arena, &self.arena[target_id]).unwrap_or(target_id)),
            arena: &mut self.arena,
        }
    }

    pub fn append_child(&mut self, target_id: NodeId, node: impl Into<DetachedNode<T>>) -> NodeId {
        let last_child = self.arena[target_id].last_child;
        let new_node = Node::new(node.into(), last_child, None, Some(target_id));
        let new_node_id = self.register(new_node);

        let target = &mut self.arena[target_id];
        target.last_child = Some(new_node_id);

        if let Some(child_id) = last_child {
            self.arena[child_id].next_sibling = Some(new_node_id);
        } else {
            target.first_child = Some(new_node_id);
        }

        new_node_id
    }

    pub fn prepend_child(&mut self, target_id: NodeId, node: impl Into<DetachedNode<T>>) -> NodeId {
        let first_child = self.arena[target_id].first_child;
        let new_node = Node::new(node.into(), None, first_child, Some(target_id));
        let new_node_id = self.register(new_node);

        let target = &mut self.arena[target_id];
        target.first_child = Some(new_node_id);

        if let Some(child_id) = first_child {
            self.arena[child_id].prev_sibling = Some(new_node_id);
        } else {
            target.last_child = Some(new_node_id);
        }

        new_node_id
    }

    pub fn insert_before(&mut self, target_id: NodeId, node: impl Into<DetachedNode<T>>) -> NodeId {
        let new_node_id = self.arena.next_slot_index();
        let new_node = {
            let target = &mut self.arena[target_id];
            if target.is_root() {
                panic!("Only one element on root allowed.");
            }
            let new_node = Node::new(
                node.into(),
                target.prev_sibling,
                Some(target_id),
                target.parent
            );
            target.prev_sibling = Some(new_node_id);
            new_node
        };

        match new_node.prev_sibling {
            Some(sibling_id) => {
                self.arena[sibling_id].next_sibling = Some(new_node_id);
            }
            None => {
                if let Some(parent_id) = new_node.parent {
                    self.arena[parent_id].first_child = Some(new_node_id);
                }
            }
        };

        self.register(new_node)
    }

    pub fn insert_after(&mut self, target_id: NodeId, node: impl Into<DetachedNode<T>>) -> NodeId {
        let new_node_id = self.arena.next_slot_index();
        let new_node = {
            let target = &mut self.arena[target_id];
            if target.is_root() {
                panic!("Only one element on root allowed.");
            }
            let new_node = Node::new(
                node.into(),
                Some(target_id),
                target.next_sibling,
                target.parent
            );
            target.next_sibling = Some(new_node_id);
            new_node
        };

        match new_node.next_sibling {
            Some(next_sibling_id) => {
                self.arena[next_sibling_id].prev_sibling = Some(new_node_id);
            }
            None => {
                if let Some(parent_id) = new_node.parent {
                    self.arena[parent_id].last_child = Some(new_node_id);
                }
            }
        };

        self.register(new_node)
    }

    pub fn ancestors(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        Ancestors {
            arena: &self.arena,
            next: self.arena[target_id].parent,
        }
    }

    pub fn ancestors_mut(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &mut Node<T>)> {
        AncestorsMut {
            next: self.arena[target_id].parent,
            arena: &mut self.arena,
        }
    }

    pub fn children(&self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &Node<T>)> {
        Siblings {
            arena: &self.arena,
            next: self.arena[target_id].first_child,
        }
    }

    pub fn children_mut(&mut self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &mut Node<T>)> {
        SiblingsMut {
            next: self.arena[target_id].first_child,
            arena: &mut self.arena,
        }
    }

    pub fn next_siblings(&self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &Node<T>)> {
        Siblings {
            arena: &self.arena,
            next: self.arena[target_id].next_sibling,
        }
    }

    pub fn next_siblings_mut(&mut self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &mut Node<T>)> {
        SiblingsMut {
            next: self.arena[target_id].next_sibling,
            arena: &mut self.arena,
        }
    }

    pub fn prev_siblings(&self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &Node<T>)> {
        Siblings {
            arena: &self.arena,
            next: self.arena[target_id].prev_sibling,
        }.rev()
    }

    pub fn prev_siblings_mut(&mut self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &mut Node<T>)> {
        SiblingsMut {
            next: self.arena[target_id].prev_sibling,
            arena: &mut self.arena,
        }.rev()
    }

    pub fn pre_ordered_descendants(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        PreOrderedDescendants {
            arena: &self.arena,
            root_id: target_id,
            next: self.arena[target_id].first_child
        }
    }

    pub fn pre_ordered_descendants_mut(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &mut Node<T>)> {
        PreOrderedDescendantsMut {
            root_id: target_id,
            next: self.arena[target_id].first_child,
            arena: &mut self.arena,
        }
    }

    pub fn post_ordered_descendants(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        PostOrderedDescendants {
            arena: &self.arena,
            root_id: target_id,
            next: grandest_child(&self.arena, &self.arena[target_id]),
        }
    }

    pub fn post_ordered_descendants_mut(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &mut Node<T>)> {
        PostOrderedDescendantsMut {
            root_id: target_id,
            next: grandest_child(&self.arena, &self.arena[target_id]),
            arena: &mut self.arena,
        }
    }

    fn register(&mut self, node: Node<T>) -> NodeId {
        assert!(node.first_child.map(|node_id| self.arena.has(node_id)).unwrap_or(true));
        assert!(node.last_child.map(|node_id| self.arena.has(node_id)).unwrap_or(true));
        assert!(node.prev_sibling.map(|node_id| self.arena.has(node_id)).unwrap_or(true));
        assert!(node.next_sibling.map(|node_id| self.arena.has(node_id)).unwrap_or(true));
        assert!(node.parent.map(|node_id| self.arena.has(node_id)).unwrap_or(true));
        self.arena.insert(node)
    }

    pub fn format(
        &self,
        f: &mut fmt::Formatter,
        node_id: NodeId,
        format_open: &impl Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result,
        format_close: &impl Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result
    ) -> fmt::Result
        where T: fmt::Display {
        self.format_rec(f, node_id, format_open, format_close, 0)
    }

    fn format_rec(
        &self,
        f: &mut fmt::Formatter,
        node_id: NodeId,
        format_open: &impl Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result,
        format_close: &impl Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result,
        level: usize
    ) -> fmt::Result
        where T: fmt::Display {
        let indent_str = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
        let node = &self.arena[node_id];

        write!(f, "{}", indent_str)?;

        format_open(f, node_id, &node.data)?;

        if let Some(child_id) = node.first_child {
            write!(f, "\n")?;
            self.format_rec(f, child_id, format_open, format_close, level + 1)?;
            write!(f, "\n{}", indent_str)?;
        }

        format_close(f, node_id, &node.data)?;

        if let Some(child_id) = node.next_sibling {
            write!(f, "\n")?;
            self.format_rec(f, child_id, format_open, format_close, level)?;
        }

        Ok(())
    }
}

impl<T> Index<usize> for Tree<T> {
    type Output = Node<T>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.arena[index]
    }
}

impl<T> IndexMut<usize> for Tree<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.arena[index]
    }
}

impl<T> Node<T> {
    fn new(detached_node: DetachedNode<T>, prev_sibling: Option<NodeId>, next_sibling: Option<NodeId>, parent: Option<NodeId>) -> Node<T> {
        Node {
            data: detached_node.data,
            first_child: detached_node.first_child,
            last_child: detached_node.last_child,
            prev_sibling,
            next_sibling,
            parent,
        }
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child
    }

    pub fn last_child(&self) -> Option<NodeId> {
        self.last_child
    }

    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling
    }

    pub fn prev_sibling(&self) -> Option<NodeId> {
        self.prev_sibling
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> From<T> for DetachedNode<T> {
    fn from(data: T) -> DetachedNode<T> {
        DetachedNode {
            data,
            first_child: None,
            last_child: None,
        }
    }
}

impl<T> Deref for DetachedNode<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for DetachedNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

struct Ancestors<'a, T> {
    arena: &'a SlotVec<Node<T>>,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for Ancestors<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = &self.arena[node_id];
            self.next = node.parent;
            (node_id, node)
        })
    }
}

struct AncestorsMut<'a, T> {
    arena: &'a mut SlotVec<Node<T>>,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for AncestorsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = unsafe {
                (&mut self.arena[node_id] as *mut Node<T>).as_mut().unwrap()
            };
            self.next = node.parent;
            (node_id, node)
        })
    }
}

struct Siblings<'a, T> {
    arena: &'a SlotVec<Node<T>>,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for Siblings<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = &self.arena[node_id];
            self.next = node.next_sibling;
            (node_id, node)
        })
    }
}

impl<'a, T> DoubleEndedIterator for Siblings<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next.map(|node_id| {
            let node = &self.arena[node_id];
            self.next = node.prev_sibling;
            (node_id, node)
        })
    }
}

struct SiblingsMut<'a, T> {
    arena: &'a mut SlotVec<Node<T>>,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for SiblingsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = unsafe {
                (&mut self.arena[node_id] as *mut Node<T>).as_mut().unwrap()
            };
            self.next = node.next_sibling;
            (node_id, node)
        })
    }
}

impl<'a, T> DoubleEndedIterator for SiblingsMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next.map(|node_id| {
            let node = unsafe {
                (&mut self.arena[node_id] as *mut Node<T>).as_mut().unwrap()
            };
            self.next = node.prev_sibling;
            (node_id, node)
        })
    }
}

pub struct PreOrderedDescendants<'a, T> {
    arena: &'a SlotVec<Node<T>>,
    root_id: NodeId,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for PreOrderedDescendants<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = &self.arena[node_id];
            self.next = pre_ordered_next_descendant(self.arena, self.root_id, node);
            (node_id, node)
        })
    }
}

pub struct PreOrderedDescendantsMut<'a, T> {
    arena: &'a mut SlotVec<Node<T>>,
    root_id: NodeId,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for PreOrderedDescendantsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = unsafe {
                (&mut self.arena[node_id] as *mut Node<T>).as_mut().unwrap()
            };
            self.next = pre_ordered_next_descendant(self.arena, self.root_id, node);
            (node_id, node)
        })
    }
}

pub struct PostOrderedDescendants<'a, T> {
    arena: &'a SlotVec<Node<T>>,
    root_id: NodeId,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for PostOrderedDescendants<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = &self.arena[node_id];
            self.next = post_ordered_next_descendant(self.arena, self.root_id, node);
            (node_id, node)
        })
    }
}

pub struct PostOrderedDescendantsMut<'a, T> {
    arena: &'a mut SlotVec<Node<T>>,
    root_id: NodeId,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for PostOrderedDescendantsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = unsafe {
                (&mut self.arena[node_id] as *mut Node<T>).as_mut().unwrap()
            };
            self.next = post_ordered_next_descendant(self.arena, self.root_id, node);
            (node_id, node)
        })
    }
}

pub struct DetachSubtree<'a, T> {
    arena: &'a mut SlotVec<Node<T>>,
    root_id: NodeId,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for DetachSubtree<'a, T> {
    type Item = (NodeId, Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = self.arena.remove(node_id);
            if node_id == self.root_id {
                if !node.is_root() {
                    // The root node cannot be detach.
                    detach_node(self.arena, &node);
                }
                self.next = None;
            } else {
                self.next = Some(post_ordered_next_descendant(self.arena, self.root_id, &node).unwrap_or(self.root_id));
            }
            (node_id, node)
        })
    }
}

impl<'a, T> Drop for DetachSubtree<'a, T> {
    fn drop(&mut self) {
        while self.next().is_some() {
        }
    }
}

fn pre_ordered_next_descendant<T>(arena: &SlotVec<Node<T>>, root_id: NodeId, node: &Node<T>) -> Option<NodeId> {
    if let Some(first_child) = node.first_child {
        Some(first_child)
    } else if let Some(next_sibling) = node.next_sibling {
        Some(next_sibling)
    } else {
        let mut parent = node.parent;
        let mut result = None;
        while let Some(parent_id) = parent {
            if parent_id == root_id {
                break;
            }
            let parent_node = &arena[parent_id];
            if let Some(sibling_id) = parent_node.next_sibling {
                result = Some(sibling_id);
                break;
            }
            parent = parent_node.parent;
        }
        result
    }
}

fn post_ordered_next_descendant<T>(arena: &SlotVec<Node<T>>, root_id: NodeId, node: &Node<T>) -> Option<NodeId> {
    if let Some(next_sibling_id) = node.next_sibling() {
        if let Some(grandest_child_id) = grandest_child(arena, &arena[next_sibling_id]) {
            Some(grandest_child_id)
        } else {
            Some(next_sibling_id)
        }
    } else {
        node.parent.filter(|&parent_id| parent_id != root_id)
    }
}

fn grandest_child<T>(arena: &SlotVec<Node<T>>, node: &Node<T>) -> Option<NodeId> {
    let mut next = node.first_child();
    let mut grandest_child = None;

    while let Some(child_id) = next {
        next = arena[child_id].first_child();
        grandest_child = Some(child_id);
    }

    grandest_child
}

fn detach_node<T>(arena: &mut SlotVec<Node<T>>, node: &Node<T>) {
    match (node.prev_sibling, node.next_sibling) {
        (Some(prev_sibling_id), Some(next_sibling_id)) => {
            arena[next_sibling_id].prev_sibling = Some(prev_sibling_id);
            arena[prev_sibling_id].next_sibling = Some(next_sibling_id);
        }
        (Some(prev_sibling_id), None) => {
            if let Some(parent_id) = node.parent {
                arena[parent_id].last_child = Some(prev_sibling_id);
            }
            arena[prev_sibling_id].next_sibling = None;
        }
        (None, Some(next_sibling_id)) => {
            if let Some(parent_id) = node.parent {
                arena[parent_id].first_child = Some(next_sibling_id);
            }
            arena[next_sibling_id].prev_sibling = None;
        }
        (None, None) => {
            if let Some(parent_id) = node.parent {
                let parent = &mut arena[parent_id];
                parent.first_child = None;
                parent.last_child = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_attached() {
        let mut tree = Tree::new();
        assert!(!tree.is_attached(0));

        let root = tree.attach("root");
        assert!(tree.is_attached(root));
    }

    #[test]
    fn test_is_root() {
        let mut tree = Tree::new();

        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(root, "bar");

        assert_eq!(tree[root].is_root(), true);
        assert_eq!(tree[foo].is_root(), false);
        assert_eq!(tree[bar].is_root(), false);
    }

    #[test]
    fn test_append_child() {
        let mut tree = Tree::new();
        let root = tree.attach("root");

        assert_eq!(tree[root], Node {
            data: "root",
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });

        let foo = tree.append_child(root, "foo");

        assert_eq!(tree[root], Node {
            data: "root",
            first_child: Some(foo),
            last_child: Some(foo),
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            parent: Some(root),
        });

        let bar = tree.append_child(root, "bar");

        assert_eq!(tree[root], Node {
            data: "root",
            first_child: Some(foo),
            last_child: Some(bar),
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: Some(bar),
            parent: Some(root),
        });
        assert_eq!(tree[bar], Node {
            data: "bar",
            first_child: None,
            last_child: None,
            prev_sibling: Some(foo),
            next_sibling: None,
            parent: Some(root),
        });
    }

    #[test]
    fn test_prepend_child() {
        let mut tree = Tree::new();
        let root = tree.attach("root");

        assert_eq!(tree[root], Node {
            data: "root",
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });

        let foo = tree.prepend_child(root, "foo");

        assert_eq!(tree[root], Node {
            data: "root",
            first_child: Some(foo),
            last_child: Some(foo),
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            parent: Some(root),
        });

        let bar = tree.prepend_child(root, "bar");

        assert_eq!(tree[root], Node {
            data: "root",
            first_child: Some(bar),
            last_child: Some(foo),
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            first_child: None,
            last_child: None,
            prev_sibling: Some(bar),
            next_sibling: None,
            parent: Some(root),
        });
        assert_eq!(tree[bar], Node {
            data: "bar",
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: Some(foo),
            parent: Some(root),
        });
    }

    #[test]
    fn test_insert_before() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(root, "bar");
        let baz = tree.insert_before(foo, "baz");
        let qux = tree.insert_before(foo, "qux");

        assert_eq!(tree[root], Node {
            data: "root",
            first_child: Some(baz),
            last_child: Some(bar),
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            first_child: None,
            last_child: None,
            prev_sibling: Some(qux),
            next_sibling: Some(bar),
            parent: Some(root),
        });
        assert_eq!(tree[bar], Node {
            data: "bar",
            first_child: None,
            last_child: None,
            prev_sibling: Some(foo),
            next_sibling: None,
            parent: Some(root),
        });
        assert_eq!(tree[baz], Node {
            data: "baz",
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: Some(qux),
            parent: Some(root),
        });
        assert_eq!(tree[qux], Node {
            data: "qux",
            first_child: None,
            last_child: None,
            prev_sibling: Some(baz),
            next_sibling: Some(foo),
            parent: Some(root),
        });
    }

    #[should_panic]
    #[test]
    fn test_insert_before_should_panic() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        tree.insert_before(root, "foo");
    }

    #[test]
    fn test_insert_after() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(root, "bar");
        let baz = tree.insert_after(bar, "baz");
        let qux = tree.insert_after(bar, "qux");

        assert_eq!(tree[root], Node {
            data: "root",
            first_child: Some(foo),
            last_child: Some(baz),
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: Some(bar),
            parent: Some(root),
        });
        assert_eq!(tree[bar], Node {
            data: "bar",
            first_child: None,
            last_child: None,
            prev_sibling: Some(foo),
            next_sibling: Some(qux),
            parent: Some(root),
        });
        assert_eq!(tree[baz], Node {
            data: "baz",
            first_child: None,
            last_child: None,
            prev_sibling: Some(qux),
            next_sibling: None,
            parent: Some(root),
        });
        assert_eq!(tree[qux], Node {
            data: "qux",
            first_child: None,
            last_child: None,
            prev_sibling: Some(bar),
            next_sibling: Some(baz),
            parent: Some(root),
        });
    }

    #[should_panic]
    #[test]
    fn test_insert_after_should_panic() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        tree.insert_after(root, "foo");
    }

    #[test]
    fn test_detach_subtree() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(foo, "bar");
        let baz = tree.append_child(bar, "baz");
        let qux = tree.append_child(foo, "qux");
        let quux = tree.append_child(root, "quux");

        assert_eq!(tree.detach_subtree(foo).collect::<Vec<_>>(), [
            (baz, Node {
                data: "baz",
                first_child: None,
                last_child: None,
                prev_sibling: None,
                next_sibling: None,
                parent: Some(bar),
            }),
            (bar, Node {
                data: "bar",
                first_child: Some(baz),
                last_child: Some(baz),
                prev_sibling: None,
                next_sibling: Some(qux),
                parent: Some(foo),
            }),
            (qux, Node {
                data: "qux",
                first_child: None,
                last_child: None,
                prev_sibling: Some(bar),
                next_sibling: None,
                parent: Some(foo),
            }),
            (foo, Node {
                data: "foo",
                first_child: Some(bar),
                last_child: Some(qux),
                prev_sibling: None,
                next_sibling: Some(quux),
                parent: Some(root),
            }),
        ]);
        assert_eq!(tree[root], Node {
            data: "root",
            first_child: Some(quux),
            last_child: Some(quux),
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[quux], Node {
            data: "quux",
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            parent: Some(root),
        });
        assert!(!tree.is_attached(foo));
        assert!(!tree.is_attached(bar));
        assert!(!tree.is_attached(baz));
        assert!(!tree.is_attached(qux));

        assert_eq!(tree.detach_subtree(root).collect::<Vec<_>>(), [
            (quux, Node {
                data: "quux",
                first_child: None,
                last_child: None,
                prev_sibling: None,
                next_sibling: None,
                parent: Some(root),
            }),
            (root, Node {
                data: "root",
                first_child: Some(quux),
                last_child: Some(quux),
                prev_sibling: None,
                next_sibling: None,
                parent: None,
            }),
        ]);
        assert!(!tree.is_attached(root));
        assert!(!tree.is_attached(foo));
        assert!(!tree.is_attached(bar));
        assert!(!tree.is_attached(baz));
        assert!(!tree.is_attached(qux));
        assert!(!tree.is_attached(quux));
    }

    #[test]
    fn test_ancestors() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(root, "bar");
        let baz = tree.append_child(foo, "baz");
        let qux = tree.append_child(baz, "qux");
        let quux = tree.append_child(qux, "quux");
        let corge = tree.append_child(baz, "corge");

        assert_eq!(tree.ancestors(root).collect::<Vec<_>>(), []);
        assert_eq!(tree.ancestors(foo).collect::<Vec<_>>(), [(root, &tree[root])]);
        assert_eq!(tree.ancestors(bar).collect::<Vec<_>>(), [(root, &tree[root])]);
        assert_eq!(tree.ancestors(baz).collect::<Vec<_>>(), [(foo, &tree[foo]), (root, &tree[root])]);
        assert_eq!(tree.ancestors(qux).collect::<Vec<_>>(), [(baz, &tree[baz]), (foo, &tree[foo]), (root, &tree[root])]);
        assert_eq!(tree.ancestors(quux).collect::<Vec<_>>(), [(qux, &tree[qux]), (baz, &tree[baz]), (foo, &tree[foo]), (root, &tree[root])]);
        assert_eq!(tree.ancestors(corge).collect::<Vec<_>>(), [(baz, &tree[baz]), (foo, &tree[foo]), (root, &tree[root])]);

        for node_id in &[root, foo, bar, baz, qux, quux, corge] {
            assert_eq!(
                tree.ancestors(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>(),
                tree.ancestors_mut(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_children() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(root, "bar");
        let baz = tree.append_child(foo, "baz");
        let qux = tree.append_child(baz, "qux");
        let quux = tree.append_child(qux, "quux");
        let corge = tree.append_child(baz, "corge");

        assert_eq!(tree.children(root).collect::<Vec<_>>(), [(foo, &tree[foo]), (bar, &tree[bar])]);
        assert_eq!(tree.children(foo).collect::<Vec<_>>(), [(baz, &tree[baz])]);
        assert_eq!(tree.children(bar).collect::<Vec<_>>(), []);
        assert_eq!(tree.children(baz).collect::<Vec<_>>(), [(qux, &tree[qux]), (corge, &tree[corge])]);
        assert_eq!(tree.children(qux).collect::<Vec<_>>(), [(quux, &tree[quux])]);
        assert_eq!(tree.children(quux).collect::<Vec<_>>(), []);
        assert_eq!(tree.children(corge).collect::<Vec<_>>(), []);

        for node_id in &[root, foo, bar, baz, qux, quux, corge] {
            assert_eq!(
                tree.children(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>(),
                tree.children_mut(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_siblings() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(root, "bar");
        let baz = tree.append_child(root, "baz");

        assert_eq!(tree.prev_siblings(root).collect::<Vec<_>>(), []);
        assert_eq!(tree.prev_siblings(foo).collect::<Vec<_>>(), []);
        assert_eq!(tree.prev_siblings(bar).collect::<Vec<_>>(), [(foo, &tree[foo])]);
        assert_eq!(tree.prev_siblings(baz).collect::<Vec<_>>(), [(bar, &tree[bar]), (foo, &tree[foo])]);

        assert_eq!(tree.next_siblings(root).collect::<Vec<_>>(), []);
        assert_eq!(tree.next_siblings(foo).collect::<Vec<_>>(), [(bar, &tree[bar]), (baz, &tree[baz])]);
        assert_eq!(tree.next_siblings(bar).collect::<Vec<_>>(), [(baz, &tree[baz])]);
        assert_eq!(tree.next_siblings(baz).collect::<Vec<_>>(), []);

        for node_id in &[root, foo, bar, baz] {
            assert_eq!(
                tree.prev_siblings(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>(),
                tree.prev_siblings_mut(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>()
            );
            assert_eq!(
                tree.next_siblings(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>(),
                tree.next_siblings_mut(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_pre_ordered_descendants() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(foo, "bar");
        let baz = tree.append_child(bar, "baz");
        let qux = tree.append_child(foo, "qux");
        let quux = tree.append_child(root, "qux");

        assert_eq!(tree.pre_ordered_descendants(root).collect::<Vec<_>>(), &[(foo, &tree[foo]), (bar, &tree[bar]), (baz, &tree[baz]), (qux, &tree[qux]), (quux, &tree[quux])]);
        assert_eq!(tree.pre_ordered_descendants(foo).collect::<Vec<_>>(), &[(bar, &tree[bar]), (baz, &tree[baz]), (qux, &tree[qux])]);
        assert_eq!(tree.pre_ordered_descendants(bar).collect::<Vec<_>>(), &[(baz, &tree[baz])]);
        assert_eq!(tree.pre_ordered_descendants(baz).collect::<Vec<_>>(), &[]);
        assert_eq!(tree.pre_ordered_descendants(qux).collect::<Vec<_>>(), &[]);
        assert_eq!(tree.pre_ordered_descendants(quux).collect::<Vec<_>>(), &[]);

        for node_id in &[root, foo, bar, baz, qux, quux] {
            assert_eq!(
                tree.pre_ordered_descendants(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>(),
                tree.pre_ordered_descendants_mut(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_post_ordered_descendants() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(foo, "bar");
        let baz = tree.append_child(bar, "baz");
        let qux = tree.append_child(foo, "qux");
        let quux = tree.append_child(root, "qux");

        assert_eq!(tree.post_ordered_descendants(root).collect::<Vec<_>>(), &[(baz, &tree[baz]), (bar, &tree[bar]), (qux, &tree[qux]), (foo, &tree[foo]), (quux, &tree[quux])]);
        assert_eq!(tree.post_ordered_descendants(foo).collect::<Vec<_>>(), &[(baz, &tree[baz]), (bar, &tree[bar]), (qux, &tree[qux])]);
        assert_eq!(tree.post_ordered_descendants(bar).collect::<Vec<_>>(), &[(baz, &tree[baz])]);
        assert_eq!(tree.post_ordered_descendants(baz).collect::<Vec<_>>(), &[]);
        assert_eq!(tree.post_ordered_descendants(qux).collect::<Vec<_>>(), &[]);
        assert_eq!(tree.post_ordered_descendants(quux).collect::<Vec<_>>(), &[]);

        for node_id in &[root, foo, bar, baz, qux, quux] {
            assert_eq!(
                tree.post_ordered_descendants(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>(),
                tree.post_ordered_descendants_mut(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>()
            );
        }
    }
}
