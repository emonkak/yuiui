use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub struct Reconciler<'a, Key, OldNode, NewNode> {
    old_keys: &'a [Key],
    old_nodes: &'a mut [Option<OldNode>],
    new_keys: &'a [Key],
    new_nodes: &'a mut [Option<NewNode>],

    old_head: usize,
    old_edge: usize,
    new_head: usize,
    new_edge: usize,

    new_index_to_old_node: Vec<OldNode>,
    old_key_to_index_map: Option<HashMap<&'a Key, usize>>,
    new_key_set: Option<HashSet<&'a Key>>,
}

#[derive(Debug)]
pub enum ReconcileResult<OldNode, NewNode> {
    New(NewNode),
    NewPlacement(OldNode, NewNode),
    Update(OldNode, NewNode),
    UpdatePlacement(OldNode, OldNode, NewNode),
    Deletion(OldNode),
}

impl<'a, Key, OldNode: Default + Clone, NewNode> Reconciler<'a, Key, OldNode, NewNode> {
    pub fn new(
        old_keys: &'a [Key],
        old_nodes: &'a mut [Option<OldNode>],
        new_keys: &'a [Key],
        new_nodes: &'a mut [Option<NewNode>]
    ) -> Self {
        Self {
            old_keys,
            old_nodes,
            new_keys,
            new_nodes,

            old_head: 0,
            old_edge: old_keys.len(),
            new_head: 0,
            new_edge: new_keys.len(),

            new_index_to_old_node: vec![Default::default(); new_keys.len()],
            old_key_to_index_map: None,
            new_key_set: None,
        }
    }
}

impl<'a, Key: Eq + Hash, OldNode: Copy, NewNode> Iterator for Reconciler<'a, Key, OldNode, NewNode> {
    type Item = ReconcileResult<OldNode, NewNode>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.old_head < self.old_edge && self.new_head < self.new_edge {
            let result = match (self.old_nodes[self.old_head].as_ref(), self.old_nodes[self.old_edge - 1].as_ref()) {
                (None, _) => {
                    self.old_head += 1;
                    continue;
                }
                (_, None) => {
                    self.old_edge -= 1;
                    continue;
                },
                (Some(&old_head_node), _) if self.old_keys[self.old_head] == self.new_keys[self.new_head] => {
                    let result = ReconcileResult::Update(old_head_node, self.new_nodes[self.new_head].take().unwrap());
                    self.new_index_to_old_node[self.new_head] = old_head_node;
                    self.old_head += 1;
                    self.new_head += 1;
                    result
                }
                (_, Some(&old_tail_node)) if self.old_keys[self.old_edge - 1] == self.new_keys[self.new_edge - 1] => {
                    let result = ReconcileResult::Update(old_tail_node, self.new_nodes[self.new_edge - 1].take().unwrap());
                    self.new_index_to_old_node[self.new_edge - 1] = old_tail_node;
                    self.old_edge -= 1;
                    self.new_edge -= 1;
                    result
                }
                (Some(&old_head_node), Some(&old_tail_node)) if self.old_keys[self.old_head] == self.new_keys[self.new_edge - 1] => {
                    let result = ReconcileResult::UpdatePlacement(old_head_node, old_tail_node, self.new_nodes[self.new_edge - 1].take().unwrap());
                    self.new_index_to_old_node[self.new_edge - 1] = old_head_node;
                    self.old_head += 1;
                    self.new_edge -= 1;
                    result
                }
                (Some(&old_head_node), Some(&old_tail_node)) if self.old_keys[self.old_edge - 1] == self.new_keys[self.new_head] => {
                    let result = ReconcileResult::UpdatePlacement(old_tail_node, old_head_node, self.new_nodes[self.new_head].take().unwrap());
                    self.new_index_to_old_node[self.new_head] = old_tail_node;
                    self.old_edge -= 1;
                    self.new_head += 1;
                    result
                }
                (Some(&old_head_node), Some(&old_tail_node)) => {
                    let new_key_set = match self.new_key_set.as_ref() {
                        Some(new_key_set) => new_key_set,
                        None => {
                            self.new_key_set = Some(self.new_keys.iter().collect::<HashSet<_>>());
                            self.new_key_set.as_ref().unwrap()
                        }
                    };

                    if !new_key_set.contains(&self.old_keys[self.old_head]) {
                        let result = ReconcileResult::Deletion(old_head_node);
                        self.old_head += 1;
                        result
                    } else if !new_key_set.contains(&self.old_keys[self.old_edge - 1]) {
                        let result = ReconcileResult::Deletion(old_tail_node);
                        self.old_edge -= 1;
                        result
                    } else {
                        let old_key_to_index_map = match self.old_key_to_index_map.as_ref() {
                            Some(old_key_to_index_map) => old_key_to_index_map,
                            None => {
                                let mut map = HashMap::with_capacity(self.old_keys.len());
                                for (i, key) in self.old_keys.iter().enumerate() {
                                    map.insert(key, i);
                                }
                                self.old_key_to_index_map = Some(map);
                                self.old_key_to_index_map.as_ref().unwrap()
                            }
                        };

                        let result = if let Some(old_node) = old_key_to_index_map
                            .get(&self.new_keys[self.new_head])
                            .copied()
                            .and_then(|old_index| self.old_nodes[old_index].take()) {
                            self.new_index_to_old_node[self.new_edge - 1] = old_node;
                            ReconcileResult::UpdatePlacement(old_node, old_head_node, self.new_nodes[self.new_edge - 1].take().unwrap())
                        } else {
                            ReconcileResult::NewPlacement(old_head_node, self.new_nodes[self.new_head].take().unwrap())
                        };

                        self.new_head += 1;

                        result
                    }
                }
            };
            return Some(result);
        }

        while self.new_head < self.new_edge {
            let result = if self.new_edge < self.new_nodes.len() {
                let old_node = self.new_index_to_old_node[self.new_edge];
                ReconcileResult::NewPlacement(old_node, self.new_nodes[self.new_head].take().unwrap())
            } else {
                ReconcileResult::New(self.new_nodes[self.new_head].take().unwrap())
            };
            self.new_head += 1;
            return Some(result);
        }

        while self.old_head < self.old_edge {
            if let Some(old_head_node) = self.old_nodes[self.old_head].take() {
                self.old_head += 1;
                return Some(ReconcileResult::Deletion(old_head_node));
            } else {
                self.old_head += 1;
            }
        }

        debug_assert!(self.new_nodes.iter().all(Option::is_none));

        None
    }
}
