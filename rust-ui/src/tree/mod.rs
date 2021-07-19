pub mod ancestors;
pub mod detach_subtree;
pub mod formatter;
pub mod move_position;
pub mod post_ordered_descendants;
pub mod pre_ordered_descendants;
pub mod siblings;
pub mod walk;

#[cfg(test)]
mod tests;

use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::slot_vec::SlotVec;

use self::ancestors::{Ancestors, AncestorsMut};
use self::detach_subtree::DetachSubtree;
use self::formatter::TreeFormatter;
use self::move_position::MovePosition;
use self::post_ordered_descendants::{PostOrderedDescendants, PostOrderedDescendantsMut};
use self::pre_ordered_descendants::{PreOrderedDescendants, PreOrderedDescendantsMut};
use self::siblings::{Siblings, SiblingsMut};
use self::walk::{Walk, WalkDirection, WalkFilter, WalkFilterMut, WalkMut};

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
            arena: SlotVec::new(),
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
            parent: None,
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
            parent: Some(parent_id),
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
            parent: Some(parent_id),
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
            parent: ref_link.parent,
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
            parent: ref_link.parent,
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

    pub fn detach_subtree(
        &mut self,
        target_id: NodeId,
    ) -> impl Iterator<Item = (NodeId, Link<T>)> + '_ {
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

    pub fn ancestors_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl Iterator<Item = (NodeId, &mut Link<T>)> {
        AncestorsMut {
            next: self.arena[target_id].parent,
            tree: self,
        }
    }

    pub fn children(
        &self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &Link<T>)> {
        Siblings {
            tree: self,
            next: self.arena[target_id].current.first_child,
        }
    }

    pub fn children_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &mut Link<T>)> {
        SiblingsMut {
            next: self.arena[target_id].current.first_child,
            tree: self,
        }
    }

    pub fn next_siblings(
        &self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &Link<T>)> {
        Siblings {
            tree: self,
            next: self.arena[target_id].next_sibling,
        }
    }

    pub fn next_siblings_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &mut Link<T>)> {
        SiblingsMut {
            next: self.arena[target_id].next_sibling,
            tree: self,
        }
    }

    pub fn prev_siblings(
        &self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &Link<T>)> {
        Siblings {
            tree: self,
            next: self.arena[target_id].prev_sibling,
        }
        .rev()
    }

    pub fn prev_siblings_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &mut Link<T>)> {
        SiblingsMut {
            next: self.arena[target_id].prev_sibling,
            tree: self,
        }
        .rev()
    }

    pub fn pre_ordered_descendants(
        &self,
        target_id: NodeId,
    ) -> impl Iterator<Item = (NodeId, &Link<T>)> {
        PreOrderedDescendants {
            tree: self,
            root_id: target_id,
            next: self.arena[target_id].current.first_child,
        }
    }

    pub fn pre_ordered_descendants_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl Iterator<Item = (NodeId, &mut Link<T>)> {
        PreOrderedDescendantsMut {
            root_id: target_id,
            next: self.arena[target_id].current.first_child,
            tree: self,
        }
    }

    pub fn post_ordered_descendants(
        &self,
        target_id: NodeId,
    ) -> impl Iterator<Item = (NodeId, &Link<T>)> {
        PostOrderedDescendants {
            tree: &self,
            root_id: target_id,
            next: self.grandest_child(target_id),
        }
    }

    pub fn post_ordered_descendants_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl Iterator<Item = (NodeId, &mut Link<T>)> {
        PostOrderedDescendantsMut {
            root_id: target_id,
            next: self.grandest_child(target_id),
            tree: self,
        }
    }

    pub fn walk(
        &self,
        target_id: NodeId,
    ) -> impl Iterator<Item = (NodeId, &Link<T>, WalkDirection)> {
        Walk {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
        }
    }

    pub fn walk_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl Iterator<Item = (NodeId, &mut Link<T>, WalkDirection)> {
        WalkMut {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
        }
    }

    pub fn walk_filter<F>(
        &self,
        target_id: NodeId,
        f: F,
    ) -> impl Iterator<Item = (NodeId, &Link<T>, WalkDirection)>
    where
        F: Fn(NodeId, &Link<T>) -> bool,
    {
        WalkFilter {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
            f,
        }
    }

    pub fn walk_filter_mut<F>(
        &mut self,
        target_id: NodeId,
        f: F,
    ) -> impl Iterator<Item = (NodeId, &mut Link<T>, WalkDirection)>
    where
        F: Fn(NodeId, &mut Link<T>) -> bool,
    {
        WalkFilterMut {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
            f,
        }
    }

    pub fn to_formatter<'a>(
        &'a self,
        node_id: NodeId,
        format_open: impl Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result + 'a,
        format_close: impl Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result + 'a,
    ) -> impl fmt::Display + 'a {
        TreeFormatter {
            tree: self,
            node_id,
            format_open,
            format_close,
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
