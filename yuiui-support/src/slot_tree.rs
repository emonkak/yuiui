use std::fmt;
use std::num::NonZeroUsize;
use std::ptr::NonNull;

use crate::slot_vec::SlotVec;

#[derive(Clone, Debug)]
pub struct SlotTree<T> {
    arena: SlotVec<Node<T>>,
}

impl<T> SlotTree<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        let mut arena = SlotVec::new();

        let null = arena.insert_null();
        debug_assert_eq!(null, 0);

        let root = arena.insert(Node {
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            parent: None,
            data,
        });
        debug_assert_eq!(root, NodeId::ROOT.get());

        Self { arena }
    }

    #[inline]
    pub fn root(&self) -> Cursor<T> {
        self.cursor(NodeId::ROOT)
    }

    #[inline]
    pub fn root_mut(&mut self) -> CursorMut<T> {
        self.cursor_mut(NodeId::ROOT)
    }

    #[inline]
    pub fn cursor(&self, id: NodeId) -> Cursor<T> {
        Cursor::new(id, self)
    }

    #[inline]
    pub fn cursor_mut(&mut self, id: NodeId) -> CursorMut<T> {
        CursorMut::new(id, self)
    }

    #[inline]
    pub fn try_cursor(&self, id: NodeId) -> Option<Cursor<T>> {
        Cursor::try_new(id, self)
    }

    #[inline]
    pub fn try_cursor_mut(&mut self, id: NodeId) -> Option<CursorMut<T>> {
        CursorMut::try_new(id, self)
    }

    #[inline]
    pub fn contains(&self, id: NodeId) -> bool {
        self.arena.contains(id.get())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.arena.len()
    }

    #[inline]
    pub fn next_node_id(&self) -> NodeId {
        NodeId::new(self.arena.next_slot_index())
    }

    #[inline]
    fn get(&self, id: NodeId) -> &Node<T> {
        &self.arena[id.get()]
    }

    #[inline]
    fn get_mut(&mut self, id: NodeId) -> &mut Node<T> {
        &mut self.arena[id.get()]
    }

    #[inline]
    fn try_get(&self, id: NodeId) -> Option<&Node<T>> {
        self.arena.get(id.get())
    }

    #[inline]
    fn try_get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.arena.get_mut(id.get())
    }

    fn attach_node(&mut self, node: Node<T>) -> NodeId {
        NodeId::new(self.arena.insert(node))
    }

    fn detach_node(&mut self, node: &Node<T>, detach_from: NodeId) {
        match (node.prev_sibling, node.next_sibling) {
            (Some(prev_sibling_id), Some(next_sibling_id)) => {
                self.get_mut(next_sibling_id).prev_sibling = Some(prev_sibling_id);
                self.get_mut(prev_sibling_id).next_sibling = Some(next_sibling_id);
            }
            (Some(prev_sibling_id), None) => {
                if let Some(parent_id) = node.parent {
                    if parent_id == detach_from {
                        self.get_mut(parent_id).last_child = Some(prev_sibling_id);
                    }
                }
                self.get_mut(prev_sibling_id).next_sibling = None;
            }
            (None, Some(next_sibling_id)) => {
                if let Some(parent_id) = node.parent {
                    if parent_id == detach_from {
                        self.get_mut(parent_id).first_child = Some(next_sibling_id);
                    }
                }
                self.get_mut(next_sibling_id).prev_sibling = None;
            }
            (None, None) => {
                if let Some(parent_id) = node.parent {
                    if parent_id == detach_from {
                        let parent = self.get_mut(parent_id);
                        parent.first_child = None;
                        parent.last_child = None;
                    }
                }
            }
        }
    }
}

impl<T: fmt::Display> fmt::Display for SlotTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_rec<T: fmt::Display>(
            tree: &SlotTree<T>,
            f: &mut fmt::Formatter,
            id: NodeId,
            level: usize,
        ) -> fmt::Result {
            let indent = unsafe { String::from_utf8_unchecked(vec![b' '; level * 4]) };
            let node = tree.get(id);

            write!(f, "{}{}: {}", indent, id.get(), node.data)?;

            if let Some(child_id) = node.first_child {
                write!(f, " {{\n")?;
                fmt_rec(tree, f, child_id, level + 1)?;
                write!(f, "\n{}}}", indent)?;
            }

            if let Some(sibling_id) = node.next_sibling {
                write!(f, "\n")?;
                fmt_rec(tree, f, sibling_id, level)?;
            }

            Ok(())
        }

        fmt_rec(self, f, NodeId::ROOT, 0)
    }
}

#[derive(Clone, Debug)]
pub struct Node<T> {
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    parent: Option<NodeId>,
    data: T,
}

impl<T> Node<T> {
    #[inline]
    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child
    }

    #[inline]
    pub fn last_child(&self) -> Option<NodeId> {
        self.last_child
    }

    #[inline]
    pub fn prev_sibling(&self) -> Option<NodeId> {
        self.prev_sibling
    }

    #[inline]
    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling
    }

    #[inline]
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    #[inline]
    pub fn data(&self) -> &T {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    #[inline]
    pub fn into_data(self) -> T {
        self.data
    }

    #[inline]
    pub fn without_data(&self) -> Node<()> {
        Node {
            first_child: self.first_child,
            last_child: self.last_child,
            prev_sibling: self.prev_sibling,
            next_sibling: self.next_sibling,
            parent: self.parent,
            data: (),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(NonZeroUsize);

impl NodeId {
    pub const ROOT: Self = Self(unsafe { NonZeroUsize::new_unchecked(1) });

    fn new(id: usize) -> Self {
        debug_assert!(id > 0);
        unsafe { Self(NonZeroUsize::new_unchecked(id)) }
    }

    pub fn get(&self) -> usize {
        self.0.get()
    }

    pub fn is_root(&self) -> bool {
        usize::from(self.0) == 1
    }
}

#[derive(Debug)]
pub struct Cursor<'a, T> {
    id: NodeId,
    current: &'a Node<T>,
    tree: &'a SlotTree<T>,
}

impl<'a, T> Cursor<'a, T> {
    fn new(id: NodeId, tree: &'a SlotTree<T>) -> Self {
        Self {
            id,
            current: tree.get(id),
            tree,
        }
    }

    fn try_new(id: NodeId, tree: &'a SlotTree<T>) -> Option<Self> {
        tree.try_get(id).map(|current| Self { id, current, tree })
    }

    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    pub fn current(&self) -> &'a Node<T> {
        self.current
    }

    #[inline]
    pub fn first_child(&self) -> Option<Cursor<T>> {
        self.current
            .first_child
            .map(move |id| Cursor::new(id, self.tree))
    }

    #[inline]
    pub fn last_child(&self) -> Option<Cursor<T>> {
        self.current
            .last_child
            .map(move |id| Cursor::new(id, self.tree))
    }

    #[inline]
    pub fn prev_sibling(&self) -> Option<Cursor<T>> {
        self.current
            .prev_sibling
            .map(move |id| Cursor::new(id, self.tree))
    }

    #[inline]
    pub fn next_sibling(&self) -> Option<Cursor<T>> {
        self.current
            .next_sibling
            .map(move |id| Cursor::new(id, self.tree))
    }

    #[inline]
    pub fn parent(&self) -> Option<Cursor<T>> {
        self.current
            .parent
            .map(move |id| Cursor::new(id, self.tree))
    }

    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        Ancestors {
            next: self.current.parent,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn children(&self) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        Siblings {
            next: self.current.first_child,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn siblings(&self) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        Siblings {
            next: self.current.next_sibling,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn descendants(&self) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        Descendants {
            next: self.current.first_child,
            root: self.id,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn descendants_from(&self, root: NodeId) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        Descendants {
            next: next_descendant(&self.tree, &self.current, root),
            root,
            tree: self.tree,
        }
    }
}

#[derive(Debug)]
pub struct CursorMut<'a, T> {
    id: NodeId,
    current: NonNull<Node<T>>,
    tree: &'a mut SlotTree<T>,
}

impl<'a, T> CursorMut<'a, T> {
    fn new(id: NodeId, tree: &'a mut SlotTree<T>) -> Self {
        Self {
            id,
            current: unsafe { NonNull::new_unchecked(tree.get_mut(id) as *mut _) },
            tree,
        }
    }

    fn try_new(id: NodeId, tree: &'a mut SlotTree<T>) -> Option<Self> {
        tree.try_get_mut(id)
            .map(|current| unsafe { NonNull::new_unchecked(current as *mut _) })
            .map(move |current| Self { id, current, tree })
    }

    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    pub fn current(&mut self) -> &'a mut Node<T> {
        unsafe { self.current.as_mut() }
    }

    #[inline]
    pub fn first_child(&mut self) -> Option<CursorMut<T>> {
        self.current()
            .first_child
            .map(move |id| CursorMut::new(id, self.tree))
    }

    #[inline]
    pub fn last_child(&mut self) -> Option<CursorMut<T>> {
        self.current()
            .last_child
            .map(move |id| CursorMut::new(id, self.tree))
    }

    #[inline]
    pub fn prev_sibling(&mut self) -> Option<CursorMut<T>> {
        self.current()
            .prev_sibling
            .map(move |id| CursorMut::new(id, self.tree))
    }

    #[inline]
    pub fn next_sibling(&mut self) -> Option<CursorMut<T>> {
        self.current()
            .next_sibling
            .map(move |id| CursorMut::new(id, self.tree))
    }

    #[inline]
    pub fn parent(&mut self) -> Option<CursorMut<T>> {
        self.current()
            .parent
            .map(move |id| CursorMut::new(id, self.tree))
    }

    pub fn append_child(&mut self, data: T) -> NodeId {
        let new_id = self.tree.next_node_id();
        let current = unsafe { self.current.as_mut() };

        let new_child = Node {
            first_child: None,
            last_child: None,
            prev_sibling: current.last_child,
            next_sibling: None,
            parent: Some(self.id),
            data,
        };

        if let Some(old_id) = current.last_child.replace(new_id) {
            self.tree.get_mut(old_id).next_sibling = Some(new_id);
        } else {
            current.first_child = Some(new_id);
        }

        self.tree.attach_node(new_child)
    }

    pub fn insert_before(&mut self, data: T) -> NodeId {
        let new_id = self.tree.next_node_id();
        let current = unsafe { self.current.as_mut() };

        if current.parent.is_none() {
            panic!("Cannot insert a node on before of the root.");
        }

        let new_sibling = Node {
            first_child: None,
            last_child: None,
            prev_sibling: current.prev_sibling,
            next_sibling: Some(self.id),
            parent: current.parent,
            data,
        };

        if let Some(old_id) = current.prev_sibling.replace(new_id) {
            self.tree.get_mut(old_id).next_sibling = Some(new_id);
        } else if let Some(parent_id) = new_sibling.parent {
            self.tree.get_mut(parent_id).first_child = Some(new_id);
        }

        self.tree.attach_node(new_sibling)
    }

    pub fn move_before(&mut self, destination_id: NodeId) {
        let current = unsafe { self.current.as_mut() };
        let parent_id = current.parent.expect("Cannot move the root.");

        self.tree.detach_node(current, parent_id);

        let destination = self.tree.get_mut(destination_id);

        current.next_sibling = Some(destination_id);
        current.parent = destination.parent;

        if let Some(prev_sibling_id) = destination.prev_sibling.replace(self.id) {
            current.prev_sibling = Some(prev_sibling_id);
            self.tree.get_mut(prev_sibling_id).next_sibling = Some(self.id);
        } else {
            current.prev_sibling = None;
            if let Some(parent_id) = destination.parent {
                self.tree.get_mut(parent_id).first_child = Some(self.id);
            }
        }
    }

    pub fn move_after(&mut self, destination_id: NodeId) {
        let current = unsafe { self.current.as_mut() };
        let parent_id = current.parent.expect("Cannot move the root.");

        self.tree.detach_node(current, parent_id);

        let destination = self.tree.get_mut(destination_id);

        current.prev_sibling = Some(destination_id);
        current.parent = destination.parent;

        if let Some(next_sibling_id) = destination.next_sibling.replace(self.id) {
            current.next_sibling = Some(next_sibling_id);
            self.tree.get_mut(next_sibling_id).prev_sibling = Some(self.id);
        } else {
            current.next_sibling = None;
            if let Some(parent_id) = destination.parent {
                self.tree.get_mut(parent_id).last_child = Some(self.id);
            }
        }
    }

    #[inline]
    pub fn ancestors(&mut self) -> impl Iterator<Item = (NodeId, &mut Node<T>)> {
        AncestorsMut {
            next: self.current().parent,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn children(&mut self) -> impl Iterator<Item = (NodeId, &mut Node<T>)> {
        SiblingsMut {
            next: self.current().first_child,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn siblings(&mut self) -> impl Iterator<Item = (NodeId, &mut Node<T>)> {
        SiblingsMut {
            next: self.current().next_sibling,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn descendants(&mut self) -> impl Iterator<Item = (NodeId, &mut Node<T>)> {
        DescendantsMut {
            next: self.current().first_child,
            root: self.id,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn descendants_from(
        &mut self,
        root: NodeId,
    ) -> impl Iterator<Item = (NodeId, &mut Node<T>)> {
        DescendantsMut {
            next: next_descendant(&self.tree, unsafe { self.current.as_ref() }, root),
            root,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn drain_descendants(&mut self) -> impl Iterator<Item = (NodeId, Node<T>)> + '_ {
        let next_stack = self
            .current()
            .first_child
            .map(|child_id| vec![child_id])
            .unwrap_or_default();
        DrainDescendants {
            next_stack,
            root: self.id,
            tree: self.tree,
        }
    }

    #[inline]
    pub fn drain_subtree(mut self) -> impl Iterator<Item = (NodeId, Node<T>)> + 'a {
        let root = self.current().parent.expect("Cannot detach the root.");
        DrainSubtree {
            next_stack: vec![self.id],
            root,
            tree: self.tree,
        }
    }
}

pub struct Ancestors<'a, T> {
    next: Option<NodeId>,
    tree: &'a SlotTree<T>,
}

pub struct AncestorsMut<'a, T> {
    next: Option<NodeId>,
    tree: &'a mut SlotTree<T>,
}

impl<'a, T> Iterator for Ancestors<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = self.tree.get(id);
            self.next = node.next_sibling;
            (id, node)
        })
    }
}

impl<'a, T> Iterator for AncestorsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = unsafe { (self.tree.get_mut(id) as *mut Node<T>).as_mut().unwrap() };
            self.next = node.parent;
            (id, node)
        })
    }
}

pub struct Siblings<'a, T> {
    next: Option<NodeId>,
    tree: &'a SlotTree<T>,
}

pub struct SiblingsMut<'a, T> {
    next: Option<NodeId>,
    tree: &'a mut SlotTree<T>,
}

impl<'a, T> Iterator for Siblings<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = self.tree.get(id);
            self.next = node.next_sibling;
            (id, node)
        })
    }
}

impl<'a, T> Iterator for SiblingsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = unsafe { (self.tree.get_mut(id) as *mut Node<T>).as_mut().unwrap() };
            self.next = node.next_sibling;
            (id, node)
        })
    }
}

pub struct Descendants<'a, T> {
    next: Option<NodeId>,
    root: NodeId,
    tree: &'a SlotTree<T>,
}

pub struct DescendantsMut<'a, T> {
    next: Option<NodeId>,
    root: NodeId,
    tree: &'a mut SlotTree<T>,
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = self.tree.get(id);
            self.next = next_descendant(self.tree, node, self.root);
            (id, node)
        })
    }
}

impl<'a, T> Iterator for DescendantsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = unsafe { (self.tree.get_mut(id) as *mut Node<T>).as_mut().unwrap() };
            self.next = next_descendant(self.tree, node, self.root);
            (id, node)
        })
    }
}

fn next_descendant<T>(tree: &SlotTree<T>, node: &Node<T>, root: NodeId) -> Option<NodeId> {
    if let Some(next_id) = node.first_child {
        Some(next_id)
    } else if let Some(next_id) = node.next_sibling {
        Some(next_id)
    } else {
        let mut current = node;
        loop {
            if let Some(sibling_id) = current.next_sibling() {
                break Some(sibling_id);
            }
            match current.parent() {
                Some(parent_id) if parent_id != root => current = tree.get(parent_id),
                _ => break None,
            }
        }
    }
}

pub struct DrainDescendants<'a, T> {
    next_stack: Vec<NodeId>,
    root: NodeId,
    tree: &'a mut SlotTree<T>,
}

impl<'a, T> Iterator for DrainDescendants<'a, T> {
    type Item = (NodeId, Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_stack.pop().map(|id| {
            let node = self.tree.arena.remove(id.get());
            self.tree.detach_node(&node, self.root);
            if let Some(next_id) = node.next_sibling {
                self.next_stack.push(next_id)
            }
            if let Some(next_id) = node.first_child {
                self.next_stack.push(next_id)
            }
            (id, node)
        })
    }
}

impl<'a, T> Drop for DrainDescendants<'a, T> {
    fn drop(&mut self) {
        while self.next().is_some() {}
    }
}

pub struct DrainSubtree<'a, T> {
    next_stack: Vec<NodeId>,
    root: NodeId,
    tree: &'a mut SlotTree<T>,
}

impl<'a, T> Iterator for DrainSubtree<'a, T> {
    type Item = (NodeId, Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_stack.pop().map(|id| {
            let node = self.tree.arena.remove(id.get());
            self.tree.detach_node(&node, self.root);
            if node.parent != Some(self.root) {
                if let Some(next_id) = node.next_sibling {
                    self.next_stack.push(next_id)
                }
            }
            if let Some(next_id) = node.first_child {
                self.next_stack.push(next_id)
            }
            (id, node)
        })
    }
}

impl<'a, T> Drop for DrainSubtree<'a, T> {
    fn drop(&mut self) {
        while self.next().is_some() {}
    }
}
