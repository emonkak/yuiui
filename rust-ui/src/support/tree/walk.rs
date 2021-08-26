use super::{Link, NodeId, Tree};

pub struct Walker<'a, T> {
    pub(super) tree: &'a Tree<T>,
    pub(super) root_id: NodeId,
    pub(super) next: Option<(NodeId, WalkDirection)>,
}

pub struct WalkerMut<'a, T> {
    pub(super) tree: &'a mut Tree<T>,
    pub(super) root_id: NodeId,
    pub(super) next: Option<(NodeId, WalkDirection)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalkDirection {
    Downward,
    Sideward,
    Upward,
}

impl<'a, T> Walker<'a, T> {
    pub fn next_match<F>(&mut self, filter: F) -> Option<(NodeId, &'a Link<T>, WalkDirection)>
    where
        F: Fn(NodeId, &Link<T>) -> bool,
    {
        self.next
            .take()
            .and_then(move |(mut node_id, mut direction)| {
                let mut link = &self.tree.arena[node_id];
                while match direction {
                    WalkDirection::Downward | WalkDirection::Sideward => !filter(node_id, link),
                    WalkDirection::Upward => false,
                } {
                    if let Some((next_node_id, next_direction)) =
                        walk_next_node(node_id, self.root_id, link, WalkDirection::Upward)
                    {
                        node_id = next_node_id;
                        direction = next_direction;
                        link = &self.tree.arena[node_id];
                    } else {
                        return None;
                    }
                }
                self.next = walk_next_node(node_id, self.root_id, link, direction);
                Some((node_id, link, direction))
            })
    }
}

impl<'a, T> WalkerMut<'a, T> {
    pub fn next_match<F>(&mut self, filter: F) -> Option<(NodeId, &'a mut Link<T>, WalkDirection)>
    where
        F: Fn(NodeId, &Link<T>) -> bool,
    {
        self.next
            .take()
            .and_then(move |(mut node_id, mut direction)| {
                let mut link = unsafe {
                    (&mut self.tree.arena[node_id] as *mut Link<T>)
                        .as_mut()
                        .unwrap()
                };
                while match direction {
                    WalkDirection::Downward | WalkDirection::Sideward => !filter(node_id, link),
                    WalkDirection::Upward => false,
                } {
                    if let Some((next_node_id, next_direction)) =
                        walk_next_node(node_id, self.root_id, link, WalkDirection::Upward)
                    {
                        node_id = next_node_id;
                        direction = next_direction;
                        link = unsafe {
                            (&mut self.tree.arena[node_id] as *mut Link<T>)
                                .as_mut()
                                .unwrap()
                        };
                    } else {
                        return None;
                    }
                }
                self.next = walk_next_node(node_id, self.root_id, link, direction);
                Some((node_id, link, direction))
            })
    }
}

impl<'a, T> Iterator for Walker<'a, T> {
    type Item = (NodeId, &'a Link<T>, WalkDirection);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|(node_id, direction)| {
            let link = &self.tree.arena[node_id];
            self.next = walk_next_node(node_id, self.root_id, link, direction);
            (node_id, link, direction)
        })
    }
}

impl<'a, T> Iterator for WalkerMut<'a, T> {
    type Item = (NodeId, &'a mut Link<T>, WalkDirection);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|(node_id, direction)| {
            let link = unsafe {
                (&mut self.tree.arena[node_id] as *mut Link<T>)
                    .as_mut()
                    .unwrap()
            };
            self.next = walk_next_node(node_id, self.root_id, link, direction);
            (node_id, link, direction)
        })
    }
}

pub fn walk_next_node<T>(
    node_id: NodeId,
    root_id: NodeId,
    link: &Link<T>,
    direction: WalkDirection,
) -> Option<(NodeId, WalkDirection)> {
    if node_id == root_id {
        match direction {
            WalkDirection::Downward => link
                .first_child()
                .map(|child_id| (child_id, WalkDirection::Downward)),
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
            }
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