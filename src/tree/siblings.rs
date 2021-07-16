use super::{Link, NodeId, Tree};

pub struct Siblings<'a, T> {
    pub(super) tree: &'a Tree<T>,
    pub(super) next: Option<NodeId>,
}

pub struct SiblingsMut<'a, T> {
    pub(super) tree: &'a mut Tree<T>,
    pub(super) next: Option<NodeId>,
}

impl<'a, T> Iterator for Siblings<'a, T> {
    type Item = (NodeId, &'a Link<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let link = &self.tree.arena[node_id];
            self.next = link.next_sibling;
            (node_id, link)
        })
    }
}

impl<'a, T> DoubleEndedIterator for Siblings<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next.map(|node_id| {
            let link = &self.tree.arena[node_id];
            self.next = link.prev_sibling;
            (node_id, link)
        })
    }
}

impl<'a, T> Iterator for SiblingsMut<'a, T> {
    type Item = (NodeId, &'a mut Link<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let link = unsafe {
                (&mut self.tree.arena[node_id] as *mut Link<T>)
                    .as_mut()
                    .unwrap()
            };
            self.next = link.next_sibling;
            (node_id, link)
        })
    }
}

impl<'a, T> DoubleEndedIterator for SiblingsMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next.map(|node_id| {
            let link = unsafe {
                (&mut self.tree.arena[node_id] as *mut Link<T>)
                    .as_mut()
                    .unwrap()
            };
            self.next = link.prev_sibling;
            (node_id, link)
        })
    }
}
