use std::fmt;
use std::num::NonZeroUsize;
use std::ops::{Index, IndexMut};
use std::ptr::NonNull;

use crate::vec::{Key, SlotVec};

#[derive(Clone, Debug)]
pub struct Node<T> {
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub parent: Option<NodeId>,
    pub data: T,
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(NonZeroUsize);

impl NodeId {
    pub const ROOT: Self = Self(unsafe { NonZeroUsize::new_unchecked(1) });

    fn new(key: Key) -> Self {
        assert!(key > 0);
        Self(unsafe { NonZeroUsize::new_unchecked(key) })
    }
}

impl Into<NonZeroUsize> for NodeId {
    fn into(self) -> NonZeroUsize {
        self.0
    }
}

impl Into<usize> for NodeId {
    fn into(self) -> usize {
        self.0.get()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct SlotGraph<T> {
    arena: SlotVec<Node<T>>,
}

impl<T> SlotGraph<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        let mut arena = SlotVec::new();

        let null = arena.reserve_key();
        debug_assert_eq!(null, 0);

        let root = arena.push(Node {
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            parent: None,
            data,
        });
        debug_assert_eq!(root, 1);

        Self { arena }
    }

    #[inline]
    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.arena.get(id.into())
    }

    #[inline]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.arena.get_mut(id.into())
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
    pub fn display(&self, id: NodeId) -> Display<T>
    where
        T: fmt::Debug,
    {
        Display::new(id, self)
    }

    #[inline]
    pub fn contains(&self, id: NodeId) -> bool {
        self.arena.contains_key(id.into())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.arena.len()
    }

    #[inline]
    pub fn next_id(&self) -> NodeId {
        NodeId::new(self.arena.next_key())
    }

    fn attach_node(&mut self, node: Node<T>) -> NodeId {
        NodeId::new(self.arena.push(node))
    }

    fn detach_node(&mut self, node: &Node<T>, origin: NodeId) {
        match (node.prev_sibling, node.next_sibling) {
            (Some(prev_sibling), Some(next_sibling)) => {
                self[next_sibling].prev_sibling = Some(prev_sibling);
                self[prev_sibling].next_sibling = Some(next_sibling);
            }
            (Some(prev_sibling), None) => {
                if let Some(parent) = node.parent {
                    if parent == origin {
                        self[parent].last_child = Some(prev_sibling);
                    }
                }
                self[prev_sibling].next_sibling = None;
            }
            (None, Some(next_sibling)) => {
                if let Some(parent) = node.parent {
                    if parent == origin {
                        self[parent].first_child = Some(next_sibling);
                    }
                }
                self[next_sibling].prev_sibling = None;
            }
            (None, None) => {
                if let Some(parent) = node.parent {
                    if parent == origin {
                        let parent = &mut self[parent];
                        parent.first_child = None;
                        parent.last_child = None;
                    }
                }
            }
        }
    }
}

impl<T> Index<NodeId> for SlotGraph<T> {
    type Output = Node<T>;

    #[inline]
    fn index(&self, id: NodeId) -> &Self::Output {
        &self.arena[id.into()]
    }
}

impl<T> IndexMut<NodeId> for SlotGraph<T> {
    #[inline]
    fn index_mut(&mut self, id: NodeId) -> &mut Self::Output {
        &mut self.arena[id.into()]
    }
}

pub struct Display<'a, T> {
    id: NodeId,
    graph: &'a SlotGraph<T>,
}

impl<'a, T> Display<'a, T>
where
    T: fmt::Debug,
{
    fn new(id: NodeId, graph: &'a SlotGraph<T>) -> Self {
        Self { graph, id }
    }

    fn fmt_rec(&self, f: &mut fmt::Formatter, id: NodeId, level: usize) -> fmt::Result
    where
        T: fmt::Debug,
    {
        let indent = unsafe { String::from_utf8_unchecked(vec![b' '; level * 4]) };
        let node = &self.graph[id.into()];

        write!(f, "{}{:?} @ {}", indent, node.data, id)?;

        if let Some(child) = node.first_child {
            write!(f, " {{\n")?;
            self.fmt_rec(f, child, level + 1)?;
            write!(f, "\n{}}}", indent)?;
        }

        if let Some(sibling) = node.next_sibling {
            write!(f, "\n")?;
            self.fmt_rec(f, sibling, level)?;
        }

        Ok(())
    }
}

impl<'a, T> fmt::Display for Display<'a, T>
where
    T: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_rec(f, self.id, 0)
    }
}

#[derive(Debug)]
pub struct Cursor<'a, T> {
    id: NodeId,
    current: &'a Node<T>,
    graph: &'a SlotGraph<T>,
}

impl<'a, T> Cursor<'a, T> {
    fn new(id: NodeId, graph: &'a SlotGraph<T>) -> Self {
        Self {
            id,
            current: &graph[id],
            graph,
        }
    }

    fn try_new(id: NodeId, graph: &'a SlotGraph<T>) -> Option<Self> {
        graph.get(id).map(|current| Self { id, current, graph })
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
            .map(move |id| Cursor::new(id, self.graph))
    }

    #[inline]
    pub fn last_child(&self) -> Option<Cursor<T>> {
        self.current
            .last_child
            .map(move |id| Cursor::new(id, self.graph))
    }

    #[inline]
    pub fn prev_sibling(&self) -> Option<Cursor<T>> {
        self.current
            .prev_sibling
            .map(move |id| Cursor::new(id, self.graph))
    }

    #[inline]
    pub fn next_sibling(&self) -> Option<Cursor<T>> {
        self.current
            .next_sibling
            .map(move |id| Cursor::new(id, self.graph))
    }

    #[inline]
    pub fn parent(&self) -> Option<Cursor<T>> {
        self.current
            .parent
            .map(move |id| Cursor::new(id, self.graph))
    }

    #[inline]
    pub fn ancestors(&self) -> Ancestors<T> {
        Ancestors {
            next: self.current.parent,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn children(&self) -> Siblings<T> {
        Siblings {
            next: self.current.first_child,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn siblings(&self) -> Siblings<T> {
        Siblings {
            next: self.current.next_sibling,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn descendants(&self) -> Descendants<T> {
        Descendants {
            next: self.current.first_child,
            origin: self.id,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn descendants_from(&self, origin: NodeId) -> Descendants<T> {
        Descendants {
            next: next_descendant(&self.graph, &self.current, origin),
            origin,
            graph: self.graph,
        }
    }
}

#[derive(Debug)]
pub struct CursorMut<'a, T> {
    id: NodeId,
    current: NonNull<Node<T>>,
    graph: &'a mut SlotGraph<T>,
}

impl<'a, T> CursorMut<'a, T> {
    fn new(id: NodeId, graph: &'a mut SlotGraph<T>) -> Self {
        Self {
            id,
            current: unsafe { NonNull::new_unchecked(&mut graph[id] as *mut _) },
            graph,
        }
    }

    fn try_new(id: NodeId, graph: &'a mut SlotGraph<T>) -> Option<Self> {
        match graph.get_mut(id) {
            Some(node) => {
                let current = unsafe { NonNull::new_unchecked(node as *mut _) };
                Some(Self { id, current, graph })
            }
            None => None,
        }
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
            .map(move |id| CursorMut::new(id, self.graph))
    }

    #[inline]
    pub fn last_child(&mut self) -> Option<CursorMut<T>> {
        self.current()
            .last_child
            .map(move |id| CursorMut::new(id, self.graph))
    }

    #[inline]
    pub fn prev_sibling(&mut self) -> Option<CursorMut<T>> {
        self.current()
            .prev_sibling
            .map(move |id| CursorMut::new(id, self.graph))
    }

    #[inline]
    pub fn next_sibling(&mut self) -> Option<CursorMut<T>> {
        self.current()
            .next_sibling
            .map(move |id| CursorMut::new(id, self.graph))
    }

    #[inline]
    pub fn parent(&mut self) -> Option<CursorMut<T>> {
        self.current()
            .parent
            .map(move |id| CursorMut::new(id, self.graph))
    }

    pub fn append_child(&mut self, data: T) -> NodeId {
        let new_child = self.graph.next_id();
        let current = unsafe { self.current.as_mut() };

        let node = Node {
            first_child: None,
            last_child: None,
            prev_sibling: current.last_child,
            next_sibling: None,
            parent: Some(self.id),
            data,
        };

        if let Some(sibling) = current.last_child.replace(new_child) {
            self.graph[sibling].next_sibling = Some(new_child);
        } else {
            current.first_child = Some(new_child);
        }

        self.graph.attach_node(node)
    }

    pub fn insert_before(&mut self, data: T) -> NodeId {
        let new_child = self.graph.next_id();
        let current = unsafe { self.current.as_mut() };

        if current.parent.is_none() {
            panic!("Cannot insert a node on before of the root.");
        }

        let node = Node {
            first_child: None,
            last_child: None,
            prev_sibling: current.prev_sibling,
            next_sibling: Some(self.id),
            parent: current.parent,
            data,
        };

        if let Some(sibling) = current.prev_sibling.replace(new_child) {
            self.graph[sibling].next_sibling = Some(new_child);
        } else if let Some(parent) = node.parent {
            self.graph[parent].first_child = Some(new_child);
        }

        self.graph.attach_node(node)
    }

    pub fn reorder_before(&mut self, sibling: NodeId) {
        let current = unsafe { self.current.as_mut() };
        let parent = current.parent.expect("Cannot move the root.");

        self.graph.detach_node(current, parent);

        let destination = &mut self.graph[sibling];

        current.next_sibling = Some(sibling);
        current.parent = destination.parent;

        if let Some(prev_sibling) = destination.prev_sibling.replace(self.id) {
            current.prev_sibling = Some(prev_sibling);
            self.graph[prev_sibling].next_sibling = Some(self.id);
        } else {
            current.prev_sibling = None;
            if let Some(parent) = destination.parent {
                self.graph[parent].first_child = Some(self.id);
            }
        }
    }

    pub fn reorder_after(&mut self, sibling: NodeId) {
        let current = unsafe { self.current.as_mut() };
        let parent = current.parent.expect("Cannot move the root.");

        self.graph.detach_node(current, parent);

        let destination = &mut self.graph[sibling];

        current.prev_sibling = Some(sibling);
        current.parent = destination.parent;

        if let Some(next_sibling) = destination.next_sibling.replace(self.id) {
            current.next_sibling = Some(next_sibling);
            self.graph[next_sibling].prev_sibling = Some(self.id);
        } else {
            current.next_sibling = None;
            if let Some(parent) = destination.parent {
                self.graph[parent].last_child = Some(self.id);
            }
        }
    }

    #[inline]
    pub fn ancestors(&mut self) -> AncestorsMut<T> {
        AncestorsMut {
            next: self.current().parent,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn children(&mut self) -> SiblingsMut<T> {
        SiblingsMut {
            next: self.current().first_child,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn siblings(&mut self) -> SiblingsMut<T> {
        SiblingsMut {
            next: self.current().next_sibling,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn descendants(&mut self) -> DescendantsMut<T> {
        DescendantsMut {
            next: self.current().first_child,
            origin: self.id,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn descendants_from(&mut self, origin: NodeId) -> DescendantsMut<T> {
        DescendantsMut {
            next: next_descendant(&self.graph, unsafe { self.current.as_ref() }, origin),
            origin,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn drain_descendants(&mut self) -> DrainDescendants<T> {
        let next_stack = self
            .current()
            .first_child
            .map(|child| vec![child])
            .unwrap_or_default();
        DrainDescendants {
            next_stack,
            origin: self.id,
            graph: self.graph,
        }
    }

    #[inline]
    pub fn drain_subtree(mut self) -> DrainSubtree<'a, T> {
        let origin = self.current().parent.expect("Cannot detach the root.");
        DrainSubtree {
            next_stack: vec![self.id],
            origin,
            graph: self.graph,
        }
    }
}

pub struct Ancestors<'a, T> {
    next: Option<NodeId>,
    graph: &'a SlotGraph<T>,
}

pub struct AncestorsMut<'a, T> {
    next: Option<NodeId>,
    graph: &'a mut SlotGraph<T>,
}

impl<'a, T> Iterator for Ancestors<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = &self.graph[id];
            self.next = node.next_sibling;
            (id, node)
        })
    }
}

impl<'a, T> Iterator for AncestorsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = unsafe { (&mut self.graph[id] as *mut Node<T>).as_mut().unwrap() };
            self.next = node.parent;
            (id, node)
        })
    }
}

pub struct Siblings<'a, T> {
    next: Option<NodeId>,
    graph: &'a SlotGraph<T>,
}

pub struct SiblingsMut<'a, T> {
    next: Option<NodeId>,
    graph: &'a mut SlotGraph<T>,
}

impl<'a, T> Iterator for Siblings<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = &self.graph[id];
            self.next = node.next_sibling;
            (id, node)
        })
    }
}

impl<'a, T> Iterator for SiblingsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = unsafe { (&mut self.graph[id] as *mut Node<T>).as_mut().unwrap() };
            self.next = node.next_sibling;
            (id, node)
        })
    }
}

pub struct Descendants<'a, T> {
    next: Option<NodeId>,
    origin: NodeId,
    graph: &'a SlotGraph<T>,
}

pub struct DescendantsMut<'a, T> {
    next: Option<NodeId>,
    origin: NodeId,
    graph: &'a mut SlotGraph<T>,
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = &self.graph[id];
            self.next = next_descendant(self.graph, node, self.origin);
            (id, node)
        })
    }
}

impl<'a, T> Iterator for DescendantsMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|id| {
            let node = unsafe { (&mut self.graph[id] as *mut Node<T>).as_mut().unwrap() };
            self.next = next_descendant(self.graph, node, self.origin);
            (id, node)
        })
    }
}

fn next_descendant<T>(graph: &SlotGraph<T>, node: &Node<T>, origin: NodeId) -> Option<NodeId> {
    if let Some(child) = node.first_child {
        Some(child)
    } else if let Some(sibling) = node.next_sibling {
        Some(sibling)
    } else {
        let mut current = node;
        loop {
            if let Some(sibling) = current.next_sibling {
                break Some(sibling);
            }
            match current.parent {
                Some(parent) if parent != origin => current = &graph[parent],
                _ => break None,
            }
        }
    }
}

pub struct DrainDescendants<'a, T> {
    next_stack: Vec<NodeId>,
    origin: NodeId,
    graph: &'a mut SlotGraph<T>,
}

impl<'a, T> Iterator for DrainDescendants<'a, T> {
    type Item = (NodeId, Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_stack.pop().map(|id| {
            let node = self.graph.arena.remove(id.into()).unwrap();
            self.graph.detach_node(&node, self.origin);
            if let Some(sibling) = node.next_sibling {
                self.next_stack.push(sibling)
            }
            if let Some(child) = node.first_child {
                self.next_stack.push(child)
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
    origin: NodeId,
    graph: &'a mut SlotGraph<T>,
}

impl<'a, T> Iterator for DrainSubtree<'a, T> {
    type Item = (NodeId, Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_stack.pop().map(|id| {
            let node = self.graph.arena.remove(id.into()).unwrap();
            self.graph.detach_node(&node, self.origin);
            if node.parent != Some(self.origin) {
                if let Some(sibling) = node.next_sibling {
                    self.next_stack.push(sibling)
                }
            }
            if let Some(child) = node.first_child {
                self.next_stack.push(child)
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
