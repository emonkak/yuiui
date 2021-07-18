use super::{Link, NodeId, Tree};

pub struct PostOrderedDescendants<'a, T> {
    pub(super) tree: &'a Tree<T>,
    pub(super) root_id: NodeId,
    pub(super) next: Option<NodeId>,
}

pub struct PostOrderedDescendantsMut<'a, T> {
    pub(super) tree: &'a mut Tree<T>,
    pub(super) root_id: NodeId,
    pub(super) next: Option<NodeId>,
}

impl<'a, T> Iterator for PostOrderedDescendants<'a, T> {
    type Item = (NodeId, &'a Link<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let link = &self.tree.arena[node_id];
            self.next = self.tree.next_post_ordered_descendant(self.root_id, link);
            (node_id, link)
        })
    }
}

impl<'a, T> Iterator for PostOrderedDescendantsMut<'a, T> {
    type Item = (NodeId, &'a mut Link<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let link = unsafe {
                (&mut self.tree.arena[node_id] as *mut Link<T>)
                    .as_mut()
                    .unwrap()
            };
            self.next = self.tree.next_post_ordered_descendant(self.root_id, link);
            (node_id, link)
        })
    }
}
