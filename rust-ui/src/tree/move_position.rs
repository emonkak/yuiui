use super::{NodeId, Tree};

pub struct MovePosition<'a, T> {
    pub(super) tree: &'a mut Tree<T>,
    pub(super) target_id: NodeId,
}

impl<'a, T> MovePosition<'a, T> {
    #[inline]
    pub fn append_child(self, parent_id: NodeId) -> NodeId {
        self.ensure_valid(parent_id);
        let target_link = self.tree.arena.remove(self.target_id);
        self.tree.detach_link(&target_link);
        self.tree.append_child(parent_id, target_link.current)
    }

    #[inline]
    pub fn prepend_child(self, parent_id: NodeId) -> NodeId {
        self.ensure_valid(parent_id);
        let target_link = self.tree.arena.remove(self.target_id);
        self.tree.detach_link(&target_link);
        self.tree.prepend_child(parent_id, target_link.current)
    }

    #[inline]
    pub fn insert_before(self, ref_id: NodeId) -> NodeId {
        self.ensure_valid(ref_id);
        let target_link = self.tree.arena.remove(self.target_id);
        self.tree.detach_link(&target_link);
        self.tree.insert_before(ref_id, target_link.current)
    }

    #[inline]
    pub fn insert_after(self, ref_id: NodeId) -> NodeId {
        self.ensure_valid(ref_id);
        let target_link = self.tree.arena.remove(self.target_id);
        self.tree.detach_link(&target_link);
        self.tree.insert_after(ref_id, target_link.current)
    }

    fn ensure_valid(&self, ref_id: NodeId) {
        assert_ne!(
            self.target_id, ref_id,
            "The target node and the reference node are same."
        );
        for (parent_id, _) in self.tree.ancestors(ref_id) {
            assert_ne!(
                self.target_id, parent_id,
                "The target node is a parent of reference node."
            );
        }
    }
}
