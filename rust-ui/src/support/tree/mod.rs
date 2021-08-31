mod ancestors;
mod detach_subtree;
mod move_position;
mod post_ordered_descendants;
mod pre_ordered_descendants;
mod siblings;
mod walk;

#[cfg(test)]
mod tests;

pub use walk::WalkDirection;

use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::support::slot_vec::SlotVec;

use ancestors::{Ancestors, AncestorsMut};
use detach_subtree::DetachSubtree;
use move_position::MovePosition;
use post_ordered_descendants::{PostOrderedDescendants, PostOrderedDescendantsMut};
use pre_ordered_descendants::{PreOrderedDescendants, PreOrderedDescendantsMut};
use siblings::{Siblings, SiblingsMut};
use walk::{Walker, WalkerMut};

#[derive(Clone, Debug)]
pub struct Tree<T> {
    arena: SlotVec<Link<T>>,
    version: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link<T> {
    current: Node<T>,
    prev_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    parent: Option<NodeId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<T> {
    pub data: T,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
}

pub type NodeId = usize;

impl<T> Tree<T> {
    pub const fn new() -> Self {
        Self {
            arena: SlotVec::new(),
            version: 0,
        }
    }

    #[inline]
    pub fn contains(&self, target_id: NodeId) -> bool {
        self.arena.contains(target_id)
    }

    #[inline]
    pub fn next_node_id(&self) -> NodeId {
        self.arena.next_slot_index()
    }

    #[inline]
    pub fn version(&self) -> usize {
        self.version
    }

    #[inline]
    pub fn attach(&mut self, node: impl Into<Node<T>>) -> NodeId {
        self.arena.insert(Link {
            current: node.into(),
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        })
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

    pub fn split_subtree(&mut self, target_id: NodeId) -> Self
    where
        T: Clone,
    {
        let mut arena = SlotVec::new();
        let mut current = &self.arena[target_id];

        arena.insert_at(target_id, current.clone());

        while let Some(child_id) = self.next_pre_ordered_descendant(target_id, current) {
            current = &self.arena[child_id];
            arena.insert_at(child_id, current.clone());
        }

        Self {
            arena,
            version: self.version,
        }
    }

    #[inline]
    pub fn detach(
        &mut self,
        target_id: NodeId,
    ) -> (Link<T>, impl Iterator<Item = (NodeId, Link<T>)> + '_) {
        let link = self.arena.remove(target_id);
        self.detach_link(&link);
        self.version += 1;
        let subtree = DetachSubtree {
            root_id: target_id,
            next: link
                .first_child()
                .map(|child_id| self.grandest_child(child_id).unwrap_or(child_id)),
            tree: self,
        };
        (link, subtree)
    }

    #[inline]
    pub fn move_position(&mut self, target_id: NodeId) -> MovePosition<'_, T> {
        MovePosition {
            tree: self,
            target_id: target_id,
        }
    }

    #[inline]
    pub fn ancestors(&self, target_id: NodeId) -> impl Iterator<Item = (NodeId, &Link<T>)> {
        Ancestors {
            tree: self,
            next: self.arena[target_id].parent,
        }
    }

    #[inline]
    pub fn ancestors_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl Iterator<Item = (NodeId, &mut Link<T>)> {
        AncestorsMut {
            next: self.arena[target_id].parent,
            tree: self,
        }
    }

    #[inline]
    pub fn children(
        &self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &Link<T>)> {
        Siblings {
            tree: self,
            next: self.arena[target_id].current.first_child,
        }
    }

    #[inline]
    pub fn children_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &mut Link<T>)> {
        SiblingsMut {
            next: self.arena[target_id].current.first_child,
            tree: self,
        }
    }

    #[inline]
    pub fn next_siblings(
        &self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &Link<T>)> {
        Siblings {
            tree: self,
            next: self.arena[target_id].next_sibling,
        }
    }

    #[inline]
    pub fn next_siblings_mut(
        &mut self,
        target_id: NodeId,
    ) -> impl DoubleEndedIterator<Item = (NodeId, &mut Link<T>)> {
        SiblingsMut {
            next: self.arena[target_id].next_sibling,
            tree: self,
        }
    }

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
    pub fn walk(&self, target_id: NodeId) -> Walker<T> {
        Walker {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
        }
    }

    #[inline]
    pub fn walk_mut(&mut self, target_id: NodeId) -> WalkerMut<T> {
        WalkerMut {
            tree: self,
            root_id: target_id,
            next: Some((target_id, WalkDirection::Downward)),
        }
    }

    pub fn format<FOpen, FClose>(
        &self,
        f: &mut fmt::Formatter,
        node_id: NodeId,
        format_open: FOpen,
        format_close: FClose,
    ) -> fmt::Result
    where
        FOpen: Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result,
        FClose: Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result,
    {
        self.format_rec(f, node_id, &format_open, &format_close, 0)
    }

    fn format_rec<FOpen, FClose>(
        &self,
        f: &mut fmt::Formatter,
        node_id: NodeId,
        format_open: &FOpen,
        format_close: &FClose,
        level: usize,
    ) -> fmt::Result
    where
        FOpen: Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result,
        FClose: Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result,
    {
        let indent_str = unsafe { String::from_utf8_unchecked(vec![b' '; level * 4]) };
        let link = &self.arena[node_id];

        write!(f, "{}", indent_str)?;

        (format_open)(f, node_id, &link.current.data)?;

        if let Some(child_id) = link.current.first_child {
            write!(f, "\n")?;
            self.format_rec(f, child_id, format_open, format_close, level + 1)?;
            write!(f, "\n{}", indent_str)?;
        }

        (format_close)(f, node_id, &link.current.data)?;

        if let Some(child_id) = link.next_sibling {
            write!(f, "\n")?;
            self.format_rec(f, child_id, format_open, format_close, level)?;
        }

        Ok(())
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

impl<T> Default for Tree<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
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
    #[inline]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.current.data
    }

    #[inline]
    pub fn first_child(&self) -> Option<NodeId> {
        self.current.first_child
    }

    #[inline]
    pub fn last_child(&self) -> Option<NodeId> {
        self.current.last_child
    }

    #[inline]
    pub fn has_child(&self) -> bool {
        self.current.first_child.is_some()
    }

    #[inline]
    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling
    }

    #[inline]
    pub fn prev_sibling(&self) -> Option<NodeId> {
        self.prev_sibling
    }

    #[inline]
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }
}

impl<T> Deref for Link<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.current.data
    }
}

impl<T> DerefMut for Link<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.current.data
    }
}

impl<T> From<T> for Node<T> {
    #[inline]
    fn from(data: T) -> Self {
        Self {
            data,
            first_child: None,
            last_child: None,
        }
    }
}
