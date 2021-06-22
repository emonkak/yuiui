use slot_vec::SlotVec;
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Debug)]
pub struct Tree<T> {
    arena: SlotVec<Node<T>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Node<T> {
    data: T,
    parent: NodeId,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
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

    pub fn is_root(&self, target_id: NodeId) -> bool {
        self.arena
            .get(target_id)
            .map_or(false, |node| node.parent == target_id)
    }

    pub fn attach(&mut self, data: T) -> NodeId {
        let root_id = self.arena.next_slot_index();
        let node = Node {
            data,
            parent: root_id,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
        };
        self.arena.insert(node)
    }

    pub fn detach(&mut self, target_id: NodeId) -> DetachedNode<T> {
        let node = self.arena.remove(target_id);
        // The root node cannot be detach.
        if node.parent != target_id {
            self.detach_node(&node);
        }
        DetachedNode {
            arena: &mut self.arena,
            next: node.first_child,
            root_id: target_id,
            node,
        }
    }

    fn detach_node(&mut self, node: &Node<T>) {
        match (node.prev_sibling, node.next_sibling) {
            (Some(prev_sibling_id), Some(next_sibling_id)) => {
                self.arena[next_sibling_id].prev_sibling = Some(prev_sibling_id);
                self.arena[prev_sibling_id].next_sibling = Some(next_sibling_id);
            }
            (Some(prev_sibling_id), None) => {
                let parent_id = node.parent;
                self.arena[parent_id].last_child = Some(prev_sibling_id);
                self.arena[prev_sibling_id].next_sibling = None;
            }
            (None, Some(next_sibling_id)) => {
                let parent_id = node.parent;
                self.arena[parent_id].first_child = Some(next_sibling_id);
                self.arena[next_sibling_id].prev_sibling = None;
            }
            (None, None) => {
                let parent_id = node.parent;
                let parent = &mut self.arena[parent_id];
                parent.first_child = None;
                parent.last_child = None;
            }
        }
    }

    pub fn append_child(&mut self, target_id: NodeId, data: T) -> NodeId {
        let last_child = self.arena[target_id].last_child;
        let new_node_id = self.arena.insert(Node {
            data,
            parent: target_id,
            first_child: None,
            last_child: None,
            prev_sibling: last_child,
            next_sibling: None,
        });

        let target = &mut self.arena[target_id];
        target.last_child = Some(new_node_id);

        if let Some(last_child_id) = last_child {
            self.arena[last_child_id].next_sibling = Some(new_node_id);
        } else {
            target.first_child = Some(new_node_id);
        }

        new_node_id
    }

    pub fn prepend_child(&mut self, target_id: NodeId, data: T) -> NodeId {
        let first_child = self.arena[target_id].first_child;
        let child_id = self.arena.insert(Node {
            data,
            parent: target_id,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: first_child,
        });

        let target = &mut self.arena[target_id];
        target.first_child = Some(child_id);

        if let Some(first_child_id) = first_child {
            self.arena[first_child_id].prev_sibling = Some(child_id);
        } else {
            target.last_child = Some(child_id);
        }

        child_id
    }

    pub fn insert_before(&mut self, target_id: NodeId, data: T) -> NodeId {
        let new_node_id = self.arena.next_slot_index();
        let new_node = {
            let target = &mut self.arena[target_id];
            if target_id == target.parent {
                panic!("Only one element on root allowed.");
            }
            let new_node = Node {
                data,
                parent: target.parent,
                first_child: None,
                last_child: None,
                prev_sibling: target.prev_sibling,
                next_sibling: Some(target_id),
            };
            target.prev_sibling = Some(new_node_id);
            new_node
        };

        match new_node.prev_sibling {
            Some(prev_sibling_id) => {
                self.arena[prev_sibling_id].next_sibling = Some(new_node_id);
            }
            None => {
                self.arena[new_node.parent].first_child = Some(new_node_id);
            }
        };

        self.arena.insert(new_node)
    }

    pub fn insert_after(&mut self, target_id: NodeId, data: T) -> NodeId {
        let new_node_id = self.arena.next_slot_index();
        let new_node = {
            let target = &mut self.arena[target_id];
            if target_id == target.parent {
                panic!("Only one element on root allowed.");
            }
            let new_node = Node {
                data,
                parent: target.parent,
                first_child: None,
                last_child: None,
                prev_sibling: Some(target_id),
                next_sibling: target.next_sibling,
            };
            target.next_sibling = Some(new_node_id);
            new_node
        };

        match new_node.next_sibling {
            Some(next_sibling_id) => {
                self.arena[next_sibling_id].prev_sibling = Some(new_node_id);
            }
            None => {
                self.arena[new_node.parent].last_child = Some(new_node_id);
            }
        };

        self.arena.insert(new_node)
    }

    pub fn to_string(&self, target_id: NodeId) -> String where T: ToString {
        fn step<T>(arena: &SlotVec<Node<T>>, node_id: NodeId, level: usize) -> String where T: ToString {
            let node = &arena[node_id];
            let indent_string = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
            let children_string = node.first_child
                .map(|first_child_id| format!(
                    "\n{}\n{}",
                    step(arena, first_child_id, level + 1),
                    indent_string
                ))
                .unwrap_or_default();
            let next_sibling_string = if level > 0 {
                node.next_sibling
                    .map(|next_sibling_id| format!(
                        "\n{}",
                        step(arena, next_sibling_id, level)
                    ))
                    .unwrap_or_default()
            } else {
                Default::default()
            };
            format!(
                "{}<{} data=\"{}\">{}</{}>{}",
                indent_string,
                node_id,
                node.data.to_string().replace('"', "\\\""),
                children_string,
                node_id,
                next_sibling_string
            )
        }
        step(&self.arena, target_id, 0)
    }

    pub fn parent(&self, target_id: NodeId) -> Option<NodeId> {
        let parent = self.arena[target_id].parent;
        if parent == target_id {
            None
        } else {
            Some(parent)
        }
    }

    pub fn ancestors(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Node<T>)>{
        Ancestors {
            arena: &self.arena,
            next: self.parent(target_id),
        }
    }

    pub fn ancestors_mut(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &mut Node<T>)>{
        AncestorsMut {
            next: self.parent(target_id),
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

    pub fn descendants(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        Descendants {
            arena: &self.arena,
            root_id: target_id,
            next: self.arena[target_id].first_child
        }
    }

    pub fn descendants_mut(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &mut Node<T>)> {
        DescendantsMut {
            root_id: target_id,
            next: self.arena[target_id].first_child,
            arena: &mut self.arena,
        }
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
    pub fn parent(&self) -> NodeId {
        self.parent
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

struct Ancestors<'a, T> {
    arena: &'a SlotVec<Node<T>>,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for Ancestors<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = &self.arena[node_id];
            self.next = if node.parent != node_id {
                Some(node.parent)
            } else {
                None
            };
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
            self.next = if node.parent != node_id {
                Some(node.parent)
            } else {
                None
            };
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

pub struct Descendants<'a, T> {
    arena: &'a SlotVec<Node<T>>,
    root_id: NodeId,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = &self.arena[node_id];
            self.next = next_descendant(self.arena, self.root_id, node);
            (node_id, node)
        })
    }
}

pub struct DescendantsMut<'a, T> {
    arena: &'a mut SlotVec<Node<T>>,
    root_id: NodeId,
    next: Option<NodeId>,
}

impl<'a, T> Iterator for DescendantsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = unsafe {
                (&mut self.arena[node_id] as *mut Node<T>).as_mut().unwrap()
            };
            self.next = next_descendant(self.arena, self.root_id, node);
            (node_id, node)
        })
    }
}

pub struct DetachedNode<'a, T> {
    arena: &'a mut SlotVec<Node<T>>,
    next: Option<NodeId>,
    root_id: NodeId,
    pub node: Node<T>,
}

impl<'a, T> Iterator for DetachedNode<'a, T> {
    type Item = (NodeId, Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let node = self.arena.remove(node_id);
            self.next = next_descendant(self.arena, self.root_id, &node);
            (node_id, node)
        })
    }
}

impl<'a, T> Drop for DetachedNode<'a, T> {
    fn drop(&mut self) {
        while self.next().is_some() {
        }
    }
}

fn next_descendant<T>(arena: &SlotVec<Node<T>>, root_id: NodeId, node: &Node<T>) -> Option<NodeId> {
    if let Some(first_child) = node.first_child {
        Some(first_child)
    } else if let Some(next_sibling) = node.next_sibling {
        Some(next_sibling)
    } else {
        let mut parent_id = node.parent;
        let mut result = None;
        while parent_id != root_id {
            let parent = &arena[parent_id];
            if let Some(next_sibling) = parent.next_sibling {
                result = Some(next_sibling);
                break;
            }
            parent_id = parent.parent;
        }
        result
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
        assert!(!tree.is_root(0));

        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        assert!(tree.is_root(root));
        assert!(!tree.is_root(foo));
    }

    #[test]
    fn test_append_child() {
        let mut tree = Tree::new();
        let root = tree.attach("root");

        assert_eq!(tree[root], Node {
            data: "root",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
        });

        let foo = tree.append_child(root, "foo");

        assert_eq!(tree[root], Node {
            data: "root",
            parent: root,
            first_child: Some(foo),
            last_child: Some(foo),
            prev_sibling: None,
            next_sibling: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
        });

        let bar = tree.append_child(root, "bar");

        assert_eq!(tree[root], Node {
            data: "root",
            parent: root,
            first_child: Some(foo),
            last_child: Some(bar),
            prev_sibling: None,
            next_sibling: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: Some(bar),
        });
        assert_eq!(tree[bar], Node {
            data: "bar",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: Some(foo),
            next_sibling: None,
        });
    }

    #[test]
    fn test_prepend_child() {
        let mut tree = Tree::new();
        let root = tree.attach("root");

        assert_eq!(tree[root], Node {
            data: "root",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
        });

        let foo = tree.prepend_child(root, "foo");

        assert_eq!(tree[root], Node {
            data: "root",
            parent: root,
            first_child: Some(foo),
            last_child: Some(foo),
            prev_sibling: None,
            next_sibling: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
        });

        let bar = tree.prepend_child(root, "bar");

        assert_eq!(tree[root], Node {
            data: "root",
            parent: root,
            first_child: Some(bar),
            last_child: Some(foo),
            prev_sibling: None,
            next_sibling: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: Some(bar),
            next_sibling: None,
        });
        assert_eq!(tree[bar], Node {
            data: "bar",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: Some(foo),
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
            parent: root,
            first_child: Some(baz),
            last_child: Some(bar),
            prev_sibling: None,
            next_sibling: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: Some(qux),
            next_sibling: Some(bar),
        });
        assert_eq!(tree[bar], Node {
            data: "bar",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: Some(foo),
            next_sibling: None,
        });
        assert_eq!(tree[baz], Node {
            data: "baz",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: Some(qux),
        });
        assert_eq!(tree[qux], Node {
            data: "qux",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: Some(baz),
            next_sibling: Some(foo),
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
            parent: root,
            first_child: Some(foo),
            last_child: Some(baz),
            prev_sibling: None,
            next_sibling: None,
        });
        assert_eq!(tree[foo], Node {
            data: "foo",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: Some(bar),
        });
        assert_eq!(tree[bar], Node {
            data: "bar",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: Some(foo),
            next_sibling: Some(qux),
        });
        assert_eq!(tree[baz], Node {
            data: "baz",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: Some(qux),
            next_sibling: None,
        });
        assert_eq!(tree[qux], Node {
            data: "qux",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: Some(bar),
            next_sibling: Some(baz),
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
    fn test_detach() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(foo, "bar");
        let baz = tree.append_child(foo, "baz");
        let qux = tree.append_child(root, "qux");
        let quux = tree.append_child(root, "quux");

        let detached_node = tree.detach(foo);
        assert_eq!(detached_node.node, Node {
            data: "foo",
            parent: root,
            first_child: Some(bar),
            last_child: Some(baz),
            prev_sibling: None,
            next_sibling: Some(qux),
        });
        assert_eq!(detached_node.collect::<Vec<_>>(), [
            (bar, Node {
                data: "bar",
                parent: foo,
                first_child: None,
                last_child: None,
                prev_sibling: None,
                next_sibling: Some(baz),
            }),
            (baz, Node {
                data: "baz",
                parent: foo,
                first_child: None,
                last_child: None,
                prev_sibling: Some(bar),
                next_sibling: None,
            }),
        ]);
        assert_eq!(tree[root], Node {
            data: "root",
            parent: root,
            first_child: Some(qux),
            last_child: Some(quux),
            prev_sibling: None,
            next_sibling: None,
        });
        assert_eq!(tree[qux], Node {
            data: "qux",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: Some(quux),
        });
        assert_eq!(tree[quux], Node {
            data: "quux",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: Some(qux),
            next_sibling: None,
        });
        assert!(!tree.is_attached(foo));
        assert!(!tree.is_attached(bar));
        assert!(!tree.is_attached(baz));

        let detached_node = tree.detach(quux);
        assert_eq!(detached_node.node, Node {
                data: "quux",
                parent: root,
                first_child: None,
                last_child: None,
                prev_sibling: Some(qux),
                next_sibling: None,
        });
        assert_eq!(detached_node.collect::<Vec<_>>(), []);
        assert_eq!(tree[root], Node {
            data: "root",
            parent: root,
            first_child: Some(qux),
            last_child: Some(qux),
            prev_sibling: None,
            next_sibling: None,
        });
        assert_eq!(tree[qux], Node {
            data: "qux",
            parent: root,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
        });
        assert!(!tree.is_attached(foo));
        assert!(!tree.is_attached(bar));
        assert!(!tree.is_attached(baz));
        assert!(!tree.is_attached(quux));

        let detached_node = tree.detach(root);
        assert_eq!(detached_node.node, Node {
            data: "root",
            parent: root,
            first_child: Some(qux),
            last_child: Some(qux),
            prev_sibling: None,
            next_sibling: None,
        });
        assert_eq!(detached_node.collect::<Vec<_>>(), [
            (qux, Node {
                data: "qux",
                parent: root,
                first_child: None,
                last_child: None,
                prev_sibling: None,
                next_sibling: None,
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
    fn test_descendants() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(foo, "bar");
        let baz = tree.append_child(foo, "baz");
        let qux = tree.append_child(root, "qux");

        assert_eq!(tree.descendants(root).collect::<Vec<_>>(), &[(foo, &tree[foo]), (bar, &tree[bar]), (baz, &tree[baz]), (qux, &tree[qux])]);
        assert_eq!(tree.descendants(foo).collect::<Vec<_>>(), &[(bar, &tree[bar]), (baz, &tree[baz])]);
        assert_eq!(tree.descendants(bar).collect::<Vec<_>>(), &[]);
        assert_eq!(tree.descendants(baz).collect::<Vec<_>>(), &[]);
        assert_eq!(tree.descendants(qux).collect::<Vec<_>>(), &[]);

        for node_id in &[root, foo, bar, baz, qux] {
            assert_eq!(
                tree.descendants(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>(),
                tree.descendants_mut(*node_id).map(|(index, node)| (index, node as *const _)).collect::<Vec<_>>()
            );
        }
    }
}
