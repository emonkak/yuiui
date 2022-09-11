use std::sync::Arc;

use super::{Depth, IdPathBuf};

#[derive(Debug, Clone)]
pub struct StateTree {
    node: Arc<StateNode>,
}

impl StateTree {
    pub fn new() -> Self {
        StateTree {
            node: Arc::new(StateNode::Root),
        }
    }

    pub fn new_subtree(&self, subscribers: Vec<(IdPathBuf, Depth)>) -> Self {
        StateTree {
            node: Arc::new(StateNode::Subtree(self.node.clone(), subscribers)),
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.node)
    }
}

#[derive(Debug, Clone)]
enum StateNode {
    Root,
    Subtree(Arc<Self>, Vec<(IdPathBuf, Depth)>),
}

pub struct Iter<'a> {
    state_node: &'a Arc<StateNode>,
}

impl<'a> Iter<'a> {
    fn new(state_node: &'a Arc<StateNode>) -> Self {
        Self { state_node }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a [(IdPathBuf, Depth)];

    fn next(&mut self) -> Option<Self::Item> {
        match self.state_node.as_ref() {
            StateNode::Root => None,
            StateNode::Subtree(parent, subscribers) => {
                self.state_node = parent;
                Some(subscribers)
            }
        }
    }
}
