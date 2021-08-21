use super::{Link, NodeId, Tree};

pub struct Ancestors<'a, T> {
    pub(super) tree: &'a Tree<T>,
    pub(super) next: Option<NodeId>,
}

pub struct AncestorsMut<'a, T> {
    pub(super) tree: &'a mut Tree<T>,
    pub(super) next: Option<NodeId>,
}

impl<'a, T> Iterator for Ancestors<'a, T> {
    type Item = (NodeId, &'a Link<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let link = &self.tree.arena[node_id];
            self.next = link.parent;
            (node_id, link)
        })
    }
}

impl<'a, T> Iterator for AncestorsMut<'a, T> {
    type Item = (NodeId, &'a mut Link<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let link = unsafe {
                (&mut self.tree.arena[node_id] as *mut Link<T>)
                    .as_mut()
                    .unwrap()
            };
            self.next = link.parent;
            (node_id, link)
        })
    }
}
