use super::{Link, NodeId, Tree};

pub struct Walk<'a, T> {
    pub(super) tree: &'a Tree<T>,
    pub(super) root_id: NodeId,
    pub(super) next: Option<(NodeId, WalkDirection)>,
}

impl<'a, T> Iterator for Walk<'a, T> {
    type Item = (NodeId, &'a Link<T>, WalkDirection);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|(node_id, direction)| {
            let link = &self.tree.arena[node_id];
            self.next = walk_next_node(node_id, self.root_id, link, &direction);
            (node_id, link, direction)
        })
    }
}

pub struct WalkMut<'a, T> {
    pub(super) tree: &'a mut Tree<T>,
    pub(super) root_id: NodeId,
    pub(super) next: Option<(NodeId, WalkDirection)>,
}

impl<'a, T> Iterator for WalkMut<'a, T> {
    type Item = (NodeId, &'a mut Link<T>, WalkDirection);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|(node_id, direction)| {
            let link = unsafe {
                (&mut self.tree.arena[node_id] as *mut Link<T>).as_mut().unwrap()
            };
            self.next = walk_next_node(node_id, self.root_id, link, &direction);
            (node_id, link, direction)
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WalkDirection {
    Downward,
    Sideward,
    Upward,
}

fn walk_next_node<T>(node_id: NodeId, root_id: NodeId, link: &Link<T>, direction: &WalkDirection) -> Option<(NodeId, WalkDirection)> {
    if node_id == root_id {
        match direction {
            WalkDirection::Downward => {
                link.first_child().map(|child_id| (child_id, WalkDirection::Downward))
            }
            _ => None,
        }
    } else {
        match direction {
            WalkDirection::Downward | WalkDirection::Sideward => {
                if let Some(child_id) = link.current.first_child {
                    Some((child_id, WalkDirection::Downward))
                } else if let Some(sibling_id) = link.next_sibling {
                    Some((sibling_id, WalkDirection::Sideward))
                } else if let Some(parent_id) = link.parent {
                    Some((parent_id, WalkDirection::Upward))
                } else {
                    None
                }
            },
            WalkDirection::Upward => {
                if let Some(sibling_id) = link.next_sibling {
                    Some((sibling_id, WalkDirection::Sideward))
                } else if let Some(parent_id) = link.parent {
                    Some((parent_id, WalkDirection::Upward))
                } else {
                    None
                }
            }
        }
    }
}
