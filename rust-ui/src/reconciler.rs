use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub struct Reconciler<Key, Node, Value> {
    old_keys: Vec<Key>,
    old_nodes: Vec<Option<Node>>,
    new_keys: Vec<Key>,
    new_values: Vec<Option<Value>>,
    new_nodes: Vec<Option<Node>>,

    old_head: usize,
    old_edge: usize,
    new_head: usize,
    new_edge: usize,

    old_keys_to_index_map: Option<HashMap<Key, usize>>,
    new_keys_set: Option<HashSet<Key>>,
}

#[derive(Debug)]
pub enum ReconcileResult<Node, Value> {
    New(Value),
    Insertion(Node, Value),
    Update(Node, Value),
    UpdateAndPlacement(Node, Node, Value),
    Deletion(Node),
}

impl<Key, Node, Value> Reconciler<Key, Node, Value> {
    pub fn new(
        old_keys: Vec<Key>,
        old_nodes: Vec<Option<Node>>,
        new_keys: Vec<Key>,
        new_values: Vec<Option<Value>>,
    ) -> Self {
        let mut new_nodes = Vec::with_capacity(new_keys.len());
        new_nodes.resize_with(new_keys.len(), || None);
        Self {
            old_head: 0,
            old_edge: old_keys.len(),
            new_head: 0,
            new_edge: new_keys.len(),

            old_keys_to_index_map: None,
            new_keys_set: None,

            old_keys,
            old_nodes,
            new_keys,
            new_values,
            new_nodes,
        }
    }
}

impl<Key, Node, Value> Iterator for Reconciler<Key, Node, Value>
where
    Node: Copy,
    Key: Eq + Hash + Copy,
{
    type Item = ReconcileResult<Node, Value>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.old_head < self.old_edge && self.new_head < self.new_edge {
            let result = match (
                self.old_nodes[self.old_head].is_some(),
                self.old_nodes[self.old_edge - 1].is_some(),
            ) {
                (false, _) => {
                    self.old_head += 1;
                    continue;
                }
                (_, false) => {
                    self.old_edge -= 1;
                    continue;
                }
                (true, _) if self.old_keys[self.old_head] == self.new_keys[self.new_head] => {
                    let old_head_node = self.old_nodes[self.old_head].take().unwrap();
                    let result = ReconcileResult::Update(
                        old_head_node,
                        self.new_values[self.new_head].take().unwrap(),
                    );
                    self.new_nodes[self.new_head] = Some(old_head_node);
                    self.old_head += 1;
                    self.new_head += 1;
                    result
                }
                (_, true)
                    if self.old_keys[self.old_edge - 1] == self.new_keys[self.new_edge - 1] =>
                {
                    let old_tail_node = self.old_nodes[self.old_edge - 1].take().unwrap();
                    let result = ReconcileResult::Update(
                        old_tail_node,
                        self.new_values[self.new_edge - 1].take().unwrap(),
                    );
                    self.new_nodes[self.new_edge - 1] = Some(old_tail_node);
                    self.old_edge -= 1;
                    self.new_edge -= 1;
                    result
                }
                (true, true)
                    if self.old_keys[self.old_head] == self.new_keys[self.new_edge - 1] =>
                {
                    let old_head_node = self.old_nodes[self.old_head].take().unwrap();
                    let old_tail_node = self.old_nodes[self.old_edge - 1].take().unwrap();
                    let result = ReconcileResult::UpdateAndPlacement(
                        old_head_node,
                        old_tail_node,
                        self.new_values[self.new_edge - 1].take().unwrap(),
                    );
                    self.new_nodes[self.new_edge - 1] = Some(old_head_node);
                    self.old_head += 1;
                    self.new_edge -= 1;
                    result
                }
                (true, true)
                    if self.old_keys[self.old_edge - 1] == self.new_keys[self.new_head] =>
                {
                    let old_head_node = self.old_nodes[self.old_head].take().unwrap();
                    let old_tail_node = self.old_nodes[self.old_edge - 1].take().unwrap();
                    let result = ReconcileResult::UpdateAndPlacement(
                        old_tail_node,
                        old_head_node,
                        self.new_values[self.new_head].take().unwrap(),
                    );
                    self.new_nodes[self.new_head] = Some(old_tail_node);
                    self.old_edge -= 1;
                    self.new_head += 1;
                    result
                }
                (true, true) => {
                    let new_keys_set = match self.new_keys_set.as_ref() {
                        Some(new_keys_set) => new_keys_set,
                        None => {
                            let mut new_keys_set = HashSet::with_capacity(self.new_keys.len());
                            new_keys_set.extend(&self.new_keys);
                            self.new_keys_set = Some(new_keys_set);
                            self.new_keys_set.as_ref().unwrap()
                        }
                    };

                    if !new_keys_set.contains(&self.old_keys[self.old_head]) {
                        let old_head_node = self.old_nodes[self.old_head].take().unwrap();
                        let result = ReconcileResult::Deletion(old_head_node);
                        self.old_head += 1;
                        result
                    } else if !new_keys_set.contains(&self.old_keys[self.old_edge - 1]) {
                        let old_tail_node = self.old_nodes[self.old_edge - 1].take().unwrap();
                        let result = ReconcileResult::Deletion(old_tail_node);
                        self.old_edge -= 1;
                        result
                    } else {
                        let old_keys_to_index_map = match self.old_keys_to_index_map.as_ref() {
                            Some(old_keys_to_index_map) => old_keys_to_index_map,
                            None => {
                                let mut map = HashMap::with_capacity(self.old_keys.len());
                                for (i, &key) in self.old_keys.iter().enumerate() {
                                    map.insert(key, i);
                                }
                                self.old_keys_to_index_map = Some(map);
                                self.old_keys_to_index_map.as_ref().unwrap()
                            }
                        };

                        let old_head_node = self.old_nodes[self.old_head].take().unwrap();
                        let result = if let Some(old_node) = old_keys_to_index_map
                            .get(&self.new_keys[self.new_head])
                            .copied()
                            .and_then(|old_index| self.old_nodes[old_index].take())
                        {
                            self.new_nodes[self.new_edge - 1] = Some(old_node);
                            ReconcileResult::UpdateAndPlacement(
                                old_node,
                                old_head_node,
                                self.new_values[self.new_head].take().unwrap(),
                            )
                        } else {
                            ReconcileResult::Insertion(
                                old_head_node,
                                self.new_values[self.new_head].take().unwrap(),
                            )
                        };

                        self.new_head += 1;

                        result
                    }
                }
            };
            return Some(result);
        }

        while self.new_head < self.new_edge {
            let result = if self.new_edge < self.new_values.len() {
                let old_node = self.new_nodes[self.new_edge].unwrap();
                ReconcileResult::Insertion(old_node, self.new_values[self.new_head].take().unwrap())
            } else {
                ReconcileResult::New(self.new_values[self.new_head].take().unwrap())
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

        debug_assert!(self.old_nodes.iter().all(Option::is_none));
        debug_assert!(self.new_values.iter().all(Option::is_none));

        None
    }
}
