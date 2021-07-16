use super::{Link, NodeId, Tree};

pub struct DetachSubtree<'a, T> {
    pub(super) tree: &'a mut Tree<T>,
    pub(super) root_id: NodeId,
    pub(super) next: Option<NodeId>,
}

impl<'a, T> Iterator for DetachSubtree<'a, T> {
    type Item = (NodeId, Link<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node_id| {
            let link = self.tree.arena.remove(node_id);
            if node_id == self.root_id {
                self.tree.detach_link(&link);
                self.next = None;
            } else {
                self.next = Some(self.tree.next_post_ordered_descendant(self.root_id, &link).unwrap_or(self.root_id));
            }
            (node_id, link)
        })
    }
}

impl<'a, T> Drop for DetachSubtree<'a, T> {
    fn drop(&mut self) {
        while self.next().is_some() {
        }
    }
}
