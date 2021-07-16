pub mod ancestors;
pub mod detach_subtree;
pub mod formatter;
pub mod move_position;
pub mod post_ordered_descendants;
pub mod pre_ordered_descendants;
pub mod siblings;
pub mod walk;

use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::slot_vec::SlotVec;

use self::ancestors::{Ancestors, AncestorsMut};
use self::pre_ordered_descendants::{PreOrderedDescendants, PreOrderedDescendantsMut};
use self::post_ordered_descendants::{PostOrderedDescendants, PostOrderedDescendantsMut};
use self::detach_subtree::DetachSubtree;
use self::move_position::MovePosition;
use self::siblings::{Siblings, SiblingsMut};
use self::walk::{Walk, WalkDirection, WalkFilter, WalkFilterMut, WalkMut};
use self::formatter::{TreeFormatter};

#[derive(Debug)]
pub struct Tree<T> {
    arena: Arena<T>,
}

pub type Arena<T> = SlotVec<Link<T>>;

#[derive(Debug, PartialEq, Eq)]
pub struct Link<T> {
    current: Node<T>,
    prev_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    parent: Option<NodeId>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node<T> {
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
        self.arena.contains(target_id)
    }

    pub fn next_node_id(&self) -> NodeId {
        self.arena.next_slot_index()
    }

    pub fn attach(&mut self, node: impl Into<Node<T>>) -> NodeId {
        let node = Link {
            current: node.into(),
            prev_sibling: None,
            next_sibling: None,
            parent: None
        };
        self.arena.insert(node)
    }

    pub fn append_child(&mut self, parent_id: NodeId, node: impl Into<Node<T>>) -> NodeId {
        let new_node_id = self.arena.next_slot_index();
        let parent_link = &mut self.arena[parent_id];

        let new_link = Link {
            current: node.into(),
            prev_sibling: parent_link.current.last_child,
            next_sibling: None,
            parent: Some(parent_id)
        };

        if let Some(child_id) = parent_link.current.last_child.replace(new_node_id) {
            self.arena[child_id].next_sibling = Some(new_node_id);
        } else {
            parent_link.current.first_child = Some(new_node_id);
        }

        self.arena.insert(new_link)
    }

    pub fn prepend_child(&mut self, parent_id: NodeId, node: impl Into<Node<T>>) -> NodeId {
        let new_node_id = self.arena.next_slot_index();
        let parent_link = &mut self.arena[parent_id];

        let new_link = Link {
            current: node.into(),
            prev_sibling: None,
            next_sibling: parent_link.current.first_child,
            parent: Some(parent_id)
        };

        if let Some(child_id) = parent_link.current.first_child.replace(new_node_id) {
            self.arena[child_id].prev_sibling = Some(new_node_id);
        } else {
            parent_link.current.last_child = Some(new_node_id);
        }

        self.arena.insert(new_link)
    }

    pub fn insert_before(&mut self, ref_id: NodeId, node: impl Into<Node<T>>) -> NodeId {
        let new_node_id = self.arena.next_slot_index();
        let ref_link = &mut self.arena[ref_id];

        if ref_link.is_root() {
            panic!("The reference node is the root node.");
        }

        let new_link = Link {
            current: node.into(),
            prev_sibling: ref_link.prev_sibling,
            next_sibling: Some(ref_id),
            parent: ref_link.parent
        };

        if let Some(sibling_id) = ref_link.prev_sibling.replace(new_node_id) {
            self.arena[sibling_id].next_sibling = Some(new_node_id);
        } else {
            if let Some(parent_id) = new_link.parent {
                self.arena[parent_id].current.first_child = Some(new_node_id);
            }
        }

        self.arena.insert(new_link)
    }

    pub fn insert_after(&mut self, ref_id: NodeId, node: impl Into<Node<T>>) -> NodeId {
        let new_node_id = self.arena.next_slot_index();
        let ref_link = &mut self.arena[ref_id];

        if ref_link.is_root() {
            panic!("The reference node is the root node.");
        }

        let new_link = Link {
            current: node.into(),
            prev_sibling: Some(ref_id),
            next_sibling: ref_link.next_sibling,
            parent: ref_link.parent
        };

        if let Some(sibling_id) = ref_link.next_sibling.replace(new_node_id) {
            self.arena[sibling_id].prev_sibling = Some(new_node_id);
        } else {
            if let Some(parent_id) = ref_link.parent {
                self.arena[parent_id].current.last_child = Some(new_node_id);
            }
        }

        self.arena.insert(new_link)
    }

    pub fn move_position(&mut self, target_id: NodeId) -> MovePosition<'_, T> {
        MovePosition {
            tree: self,
            target_id: target_id,
        }
    }

    pub fn detach_subtree(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, Link<T>)> + '_ {
        DetachSubtree {
            root_id: target_id,
            next: Some(self.grandest_child(target_id).unwrap_or(target_id)),
            tree: self,
        }
    }

    pub fn ancestors(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Link<T>)> {
        Ancestors {
            tree: self,
            next: self.arena[target_id].parent,
        }
    }

    pub fn ancestors_mut(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &mut Link<T>)> {
        AncestorsMut {
            next: self.arena[target_id].parent,
            tree: self,
        }
    }

    pub fn children(&self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &Link<T>)> {
        Siblings {
            tree: self,
            next: self.arena[target_id].current.first_child,
        }
    }

    pub fn children_mut(&mut self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &mut Link<T>)> {
        SiblingsMut {
            next: self.arena[target_id].current.first_child,
            tree: self,
        }
    }

    pub fn next_siblings(&self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &Link<T>)> {
        Siblings {
            tree: self,
            next: self.arena[target_id].next_sibling,
        }
    }

    pub fn next_siblings_mut(&mut self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &mut Link<T>)> {
        SiblingsMut {
            next: self.arena[target_id].next_sibling,
            tree: self,
        }
    }

    pub fn prev_siblings(&self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &Link<T>)> {
        Siblings {
            tree: self,
            next: self.arena[target_id].prev_sibling,
        }.rev()
    }

    pub fn prev_siblings_mut(&mut self, target_id: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, &mut Link<T>)> {
        SiblingsMut {
            next: self.arena[target_id].prev_sibling,
            tree: self
        }.rev()
    }

    pub fn pre_ordered_descendants(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Link<T>)> {
        PreOrderedDescendants {
            tree: self,
            root_id: target_id,
            next: self.arena[target_id].current.first_child
        }
    }

    pub fn pre_ordered_descendants_mut(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &mut Link<T>)> {
        PreOrderedDescendantsMut {
            root_id: target_id,
            next: self.arena[target_id].current.first_child,
            tree: self,
        }
    }

    pub fn post_ordered_descendants(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Link<T>)> {
        PostOrderedDescendants {
            tree: &self,
            root_id: target_id,
            next: self.grandest_child(target_id),
        }
    }

    pub fn post_ordered_descendants_mut(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &mut Link<T>)> {
        PostOrderedDescendantsMut {
            root_id: target_id,
            next: self.grandest_child(target_id),
            tree: self,
        }
    }

    pub fn walk(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Link<T>, WalkDirection)> {
        Walk {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
        }
    }

    pub fn walk_mut(&mut self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &mut Link<T>, WalkDirection)> {
        WalkMut {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
        }
    }

    pub fn walk_filter<F>(&self, target_id: NodeId, f: F) -> impl Iterator<Item = (NodeId, &Link<T>, WalkDirection)>
    where
        F: Fn(NodeId, &Link<T>) -> bool {
        WalkFilter {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
            f,
        }
    }

    pub fn walk_filter_mut<F>(&mut self, target_id: NodeId, f: F) -> impl Iterator<Item = (NodeId, &mut Link<T>, WalkDirection)>
    where
        F: Fn(NodeId, &mut Link<T>) -> bool {
        WalkFilterMut {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
            f,
        }
    }

    pub fn format<'a>(
        &'a self,
        node_id: NodeId,
        format_open: impl Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result + 'a,
        format_close: impl Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result + 'a
    ) -> impl fmt::Display + 'a where T: fmt::Display {
        TreeFormatter {
            tree: self,
            node_id,
            format_open,
            format_close
        }
    }

    fn next_pre_ordered_descendant(&self, root_id: NodeId, link: &Link<T>) -> Option<NodeId> {
        if let Some(child_id) = link.current.first_child {
            Some(child_id)
        } else if let Some(sibling_id) = link.next_sibling {
            Some(sibling_id)
        } else {
            let mut parent = link.parent;
            let mut result = None;
            while let Some(parent_id) = parent {
                if parent_id == root_id {
                    break;
                }
                let parent_node = &self.arena[parent_id];
                if let Some(sibling_id) = parent_node.next_sibling {
                    result = Some(sibling_id);
                    break;
                }
                parent = parent_node.parent;
            }
            result
        }
    }

    fn next_post_ordered_descendant(&self, root_id: NodeId, link: &Link<T>) -> Option<NodeId> {
        if let Some(sibling_id) = link.next_sibling() {
            if let Some(grandest_child_id) = self.grandest_child(sibling_id) {
                Some(grandest_child_id)
            } else {
                Some(sibling_id)
            }
        } else {
            link.parent.filter(|&parent_id| parent_id != root_id)
        }
    }

    fn grandest_child(&self, node_id: NodeId) -> Option<NodeId> {
        let mut next = self.arena[node_id].first_child();
        let mut grandest_child = None;

        while let Some(child_id) = next {
            next = self.arena[child_id].first_child();
            grandest_child = Some(child_id);
        }

        grandest_child
    }

    fn detach_link(&mut self, link: &Link<T>) {
        match (link.prev_sibling, link.next_sibling) {
            (Some(prev_sibling_id), Some(next_sibling_id)) => {
                self.arena[next_sibling_id].prev_sibling = Some(prev_sibling_id);
                self.arena[prev_sibling_id].next_sibling = Some(next_sibling_id);
            }
            (Some(prev_sibling_id), None) => {
                if let Some(parent_id) = link.parent {
                    self.arena[parent_id].current.last_child = Some(prev_sibling_id);
                }
                self.arena[prev_sibling_id].next_sibling = None;
            }
            (None, Some(next_sibling_id)) => {
                if let Some(parent_id) = link.parent {
                    self.arena[parent_id].current.first_child = Some(next_sibling_id);
                }
                self.arena[next_sibling_id].prev_sibling = None;
            }
            (None, None) => {
                if let Some(parent_id) = link.parent {
                    let parent = &mut self.arena[parent_id];
                    parent.current.first_child = None;
                    parent.current.last_child = None;
                }
            }
        }
    }
}

impl<T> Index<usize> for Tree<T> {
    type Output = Link<T>;

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

impl<T> Link<T> {
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn first_child(&self) -> Option<NodeId> {
        self.current.first_child
    }

    pub fn last_child(&self) -> Option<NodeId> {
        self.current.last_child
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

impl<T> Deref for Link<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.current.data
    }
}

impl<T> DerefMut for Link<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.current.data
    }
}

impl<T> From<T> for Node<T> {
    fn from(data: T) -> Node<T> {
        Node {
            data,
            first_child: None,
            last_child: None,
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

        assert_eq!(tree[root], Link {
            current: Node {
                data: "root",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });

        let foo = tree.append_child(root, "foo");

        assert_eq!(tree[root], Link {
            current: Node {
                data: "root",
                first_child: Some(foo),
                last_child: Some(foo),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: Some(root),
        });

        let bar = tree.append_child(root, "bar");

        assert_eq!(tree[root], Link {
            current: Node {
                data: "root",
                first_child: Some(foo),
                last_child: Some(bar),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: Some(bar),
            parent: Some(root),
        });
        assert_eq!(tree[bar], Link {
            current: Node {
                data: "bar",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(foo),
            next_sibling: None,
            parent: Some(root),
        });
    }

    #[test]
    fn test_prepend_child() {
        let mut tree = Tree::new();
        let root = tree.attach("root");

        assert_eq!(tree[root], Link {
            current: Node {
                data: "root",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });

        let foo = tree.prepend_child(root, "foo");

        assert_eq!(tree[root], Link {
            current: Node {
                data: "root",
                first_child: Some(foo),
                last_child: Some(foo),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: Some(root),
        });

        let bar = tree.prepend_child(root, "bar");

        assert_eq!(tree[root], Link {
            current: Node {
                data: "root",
                first_child: Some(bar),
                last_child: Some(foo),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(bar),
            next_sibling: None,
            parent: Some(root),
        });
        assert_eq!(tree[bar], Link {
            current: Node {
                data: "bar",
                first_child: None,
                last_child: None,
            },
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

        assert_eq!(tree[root], Link {
            current: Node {
                data: "root",
                first_child: Some(baz),
                last_child: Some(bar),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(qux),
            next_sibling: Some(bar),
            parent: Some(root),
        });
        assert_eq!(tree[bar], Link {
            current: Node {
                data: "bar",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(foo),
            next_sibling: None,
            parent: Some(root),
        });
        assert_eq!(tree[baz], Link {
            current: Node {
                data: "baz",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: Some(qux),
            parent: Some(root),
        });
        assert_eq!(tree[qux], Link {
            current: Node {
                data: "qux",
                first_child: None,
                last_child: None,
            },
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

        assert_eq!(tree[root], Link {
            current: Node {
                data: "root",
                first_child: Some(foo),
                last_child: Some(baz),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[foo], Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: Some(bar),
            parent: Some(root),
        });
        assert_eq!(tree[bar], Link {
            current: Node {
                data: "bar",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(foo),
            next_sibling: Some(qux),
            parent: Some(root),
        });
        assert_eq!(tree[baz], Link {
            current: Node {
                data: "baz",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(qux),
            next_sibling: None,
            parent: Some(root),
        });
        assert_eq!(tree[qux], Link {
            current: Node {
                data: "qux",
                first_child: None,
                last_child: None,
            },
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
    fn test_detach() {
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(foo, "bar");
        let baz = tree.append_child(bar, "baz");
        let qux = tree.append_child(foo, "qux");
        let quux = tree.append_child(root, "quux");

        assert_eq!(tree.detach_subtree(foo).collect::<Vec<_>>(), [
            (baz, Link {
                current: Node {
                    data: "baz",
                    first_child: None,
                    last_child: None,
                },
                prev_sibling: None,
                next_sibling: None,
                parent: Some(bar),
            }),
            (bar, Link {
                current: Node {
                    data: "bar",
                    first_child: Some(baz),
                    last_child: Some(baz),
                },
                prev_sibling: None,
                next_sibling: Some(qux),
                parent: Some(foo),
            }),
            (qux, Link {
                current: Node {
                    data: "qux",
                    first_child: None,
                    last_child: None,
                },
                prev_sibling: Some(bar),
                next_sibling: None,
                parent: Some(foo),
            }),
            (foo, Link {
                current: Node {
                    data: "foo",
                    first_child: Some(bar),
                    last_child: Some(qux),
                },
                prev_sibling: None,
                next_sibling: Some(quux),
                parent: Some(root),
            }),
        ]);
        assert_eq!(tree[root], Link {
            current: Node {
                data: "root",
                first_child: Some(quux),
                last_child: Some(quux),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        });
        assert_eq!(tree[quux], Link {
            current: Node {
                data: "quux",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: Some(root),
        });
        assert!(!tree.is_attached(foo));
        assert!(!tree.is_attached(bar));
        assert!(!tree.is_attached(baz));
        assert!(!tree.is_attached(qux));

        assert_eq!(tree.detach_subtree(root).collect::<Vec<_>>(), [
            (quux, Link {
                current: Node {
                    data: "quux",
                    first_child: None,
                    last_child: None,
                },
                prev_sibling: None,
                next_sibling: None,
                parent: Some(root),
            }),
            (root, Link {
                current: Node {
                    data: "root",
                    first_child: Some(quux),
                    last_child: Some(quux),
                },
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
                tree.ancestors(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>(),
                tree.ancestors_mut(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>()
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
                tree.children(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>(),
                tree.children_mut(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>()
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
                tree.prev_siblings(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>(),
                tree.prev_siblings_mut(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>()
            );
            assert_eq!(
                tree.next_siblings(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>(),
                tree.next_siblings_mut(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>()
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
                tree.pre_ordered_descendants(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>(),
                tree.pre_ordered_descendants_mut(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>()
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
                tree.post_ordered_descendants(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>(),
                tree.post_ordered_descendants_mut(*node_id).map(|(index, link)| (index, link as *const _)).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_walk() {
        //           root
        //          /   \
        //       foo    quux
        //      /   \
        //   bar    qux
        //   /
        // baz
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(foo, "bar");
        let baz = tree.append_child(bar, "baz");
        let qux = tree.append_child(foo, "qux");
        let quux = tree.append_child(root, "quux");

        assert_eq!(tree.walk(root).collect::<Vec<_>>(), &[
            (root, &tree[root], WalkDirection::Downward),
            (foo, &tree[foo], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Downward),
            (baz, &tree[baz], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Upward),
            (qux, &tree[qux], WalkDirection::Sideward),
            (foo, &tree[foo], WalkDirection::Upward),
            (quux, &tree[quux], WalkDirection::Sideward),
            (root, &tree[root], WalkDirection::Upward),
        ]);
        assert_eq!(tree.walk(foo).collect::<Vec<_>>(), &[
            (foo, &tree[foo], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Downward),
            (baz, &tree[baz], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Upward),
            (qux, &tree[qux], WalkDirection::Sideward),
            (foo, &tree[foo], WalkDirection::Upward),
        ]);
        assert_eq!(tree.walk(bar).collect::<Vec<_>>(), &[
            (bar, &tree[bar], WalkDirection::Downward),
            (baz, &tree[baz], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Upward),
        ]);
        assert_eq!(tree.walk(baz).collect::<Vec<_>>(), &[
            (baz, &tree[baz], WalkDirection::Downward),
        ]);
        assert_eq!(tree.walk(qux).collect::<Vec<_>>(), &[
            (qux, &tree[qux], WalkDirection::Downward),
        ]);
        assert_eq!(tree.walk(quux).collect::<Vec<_>>(), &[
            (quux, &tree[quux], WalkDirection::Downward),
        ]);

        for node_id in &[root, foo, bar, baz, qux, quux] {
            assert_eq!(
                tree.walk(*node_id).map(|(index, link, direction)| (index, link as *const _, direction)).collect::<Vec<_>>(),
                tree.walk_mut(*node_id).map(|(index, link, direction)| (index, link as *const _, direction)).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_walk_filter() {
        //           root
        //          /   \
        //       foo    quux
        //      /   \
        //   bar    qux
        //   /
        // baz
        let mut tree = Tree::new();
        let root = tree.attach("root");
        let foo = tree.append_child(root, "foo");
        let bar = tree.append_child(foo, "bar");
        let baz = tree.append_child(bar, "baz");
        let qux = tree.append_child(foo, "qux");
        let quux = tree.append_child(root, "quux");

        assert_eq!(tree.walk_filter(root, |node_id, _| node_id != bar).collect::<Vec<_>>(), &[
            (root, &tree[root], WalkDirection::Downward),
            (foo, &tree[foo], WalkDirection::Downward),
            (qux, &tree[qux], WalkDirection::Sideward),
            (foo, &tree[foo], WalkDirection::Upward),
            (quux, &tree[quux], WalkDirection::Sideward),
            (root, &tree[root], WalkDirection::Upward),
        ]);
        assert_eq!(tree.walk_filter(foo, |node_id, _| node_id != bar).collect::<Vec<_>>(), &[
            (foo, &tree[foo], WalkDirection::Downward),
            (qux, &tree[qux], WalkDirection::Sideward),
            (foo, &tree[foo], WalkDirection::Upward),
        ]);
        assert_eq!(tree.walk_filter(bar, |node_id, _| node_id != bar).collect::<Vec<_>>(), &[
        ]);
        assert_eq!(tree.walk_filter(baz, |node_id, _| node_id != bar).collect::<Vec<_>>(), &[
            (baz, &tree[baz], WalkDirection::Downward),
        ]);
        assert_eq!(tree.walk_filter(qux, |node_id, _| node_id != bar).collect::<Vec<_>>(), &[
            (qux, &tree[qux], WalkDirection::Downward),
        ]);
        assert_eq!(tree.walk_filter(quux, |node_id, _| node_id != bar).collect::<Vec<_>>(), &[
            (quux, &tree[quux], WalkDirection::Downward),
        ]);

        for node_id in &[root, foo, bar, baz, qux, quux] {
            assert_eq!(
                tree.walk_filter(*node_id, |node_id, _| node_id != bar).map(|(index, link, direction)| (index, link as *const _, direction)).collect::<Vec<_>>(),
                tree.walk_filter_mut(*node_id, |node_id, _| node_id != bar).map(|(index, link, direction)| (index, link as *const _, direction)).collect::<Vec<_>>()
            );
        }
    }
}
