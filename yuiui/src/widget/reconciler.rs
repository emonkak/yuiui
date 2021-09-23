use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::FusedIterator;

pub struct Reconciler<Key, Id, Value> {
    old_keys: Vec<Key>,
    old_ids: Vec<Option<Id>>,
    new_keys: Vec<Key>,
    new_ids: Vec<Option<Id>>,
    new_values: Vec<Option<Value>>,

    old_head: usize,
    old_edge: usize,
    new_head: usize,
    new_edge: usize,

    old_keys_to_index_map: Option<HashMap<Key, usize>>,
    new_keys_set: Option<HashSet<Key>>,
}

#[derive(Debug)]
pub enum ReconcileResult<Id, Value> {
    Append(Value),
    Insert(Id, Value),
    Update(Id, Value),
    UpdateAndMove(Id, Id, Value),
    Remove(Id),
}

impl<Key, Id, Value> Reconciler<Key, Id, Value> {
    pub fn new(
        old_keys: Vec<Key>,
        old_ids: Vec<Option<Id>>,
        new_keys: Vec<Key>,
        new_values: Vec<Option<Value>>,
    ) -> Self {
        let mut new_ids = Vec::with_capacity(new_keys.len());
        new_ids.resize_with(new_keys.len(), || None);
        Self {
            old_head: 0,
            old_edge: old_keys.len(),
            new_head: 0,
            new_edge: new_keys.len(),

            old_keys_to_index_map: None,
            new_keys_set: None,

            old_keys,
            old_ids,
            new_keys,
            new_values,
            new_ids,
        }
    }
}

impl<Key, Id, Value> Iterator for Reconciler<Key, Id, Value>
where
    Id: Copy,
    Key: Eq + Hash + Copy,
{
    type Item = ReconcileResult<Id, Value>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.old_head < self.old_edge && self.new_head < self.new_edge {
            let result = match (
                self.old_ids[self.old_head].is_some(),
                self.old_ids[self.old_edge - 1].is_some(),
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
                    let old_head_id = self.old_ids[self.old_head].take().unwrap();
                    let result = ReconcileResult::Update(
                        old_head_id,
                        self.new_values[self.new_head].take().unwrap(),
                    );
                    self.new_ids[self.new_head] = Some(old_head_id);
                    self.old_head += 1;
                    self.new_head += 1;
                    result
                }
                (_, true)
                    if self.old_keys[self.old_edge - 1] == self.new_keys[self.new_edge - 1] =>
                {
                    let old_tail_id = self.old_ids[self.old_edge - 1].take().unwrap();
                    let result = ReconcileResult::Update(
                        old_tail_id,
                        self.new_values[self.new_edge - 1].take().unwrap(),
                    );
                    self.new_ids[self.new_edge - 1] = Some(old_tail_id);
                    self.old_edge -= 1;
                    self.new_edge -= 1;
                    result
                }
                (true, true)
                    if self.old_keys[self.old_head] == self.new_keys[self.new_edge - 1] =>
                {
                    let old_head_id = self.old_ids[self.old_head].take().unwrap();
                    let old_tail_id = self.old_ids[self.old_edge - 1].take().unwrap();
                    let result = ReconcileResult::UpdateAndMove(
                        old_head_id,
                        old_tail_id,
                        self.new_values[self.new_edge - 1].take().unwrap(),
                    );
                    self.new_ids[self.new_edge - 1] = Some(old_head_id);
                    self.old_head += 1;
                    self.new_edge -= 1;
                    result
                }
                (true, true)
                    if self.old_keys[self.old_edge - 1] == self.new_keys[self.new_head] =>
                {
                    let old_head_id = self.old_ids[self.old_head].take().unwrap();
                    let old_tail_id = self.old_ids[self.old_edge - 1].take().unwrap();
                    let result = ReconcileResult::UpdateAndMove(
                        old_tail_id,
                        old_head_id,
                        self.new_values[self.new_head].take().unwrap(),
                    );
                    self.new_ids[self.new_head] = Some(old_tail_id);
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
                        let old_head_id = self.old_ids[self.old_head].take().unwrap();
                        let result = ReconcileResult::Remove(old_head_id);
                        self.old_head += 1;
                        result
                    } else if !new_keys_set.contains(&self.old_keys[self.old_edge - 1]) {
                        let old_tail_id = self.old_ids[self.old_edge - 1].take().unwrap();
                        let result = ReconcileResult::Remove(old_tail_id);
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

                        let old_head_id = self.old_ids[self.old_head].take().unwrap();
                        let result = if let Some(old_id) = old_keys_to_index_map
                            .get(&self.new_keys[self.new_head])
                            .copied()
                            .and_then(|old_index| self.old_ids[old_index].take())
                        {
                            self.new_ids[self.new_edge - 1] = Some(old_id);
                            ReconcileResult::UpdateAndMove(
                                old_id,
                                old_head_id,
                                self.new_values[self.new_head].take().unwrap(),
                            )
                        } else {
                            ReconcileResult::Insert(
                                old_head_id,
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
                let old_id = self.new_ids[self.new_edge].unwrap();
                ReconcileResult::Insert(old_id, self.new_values[self.new_head].take().unwrap())
            } else {
                ReconcileResult::Append(self.new_values[self.new_head].take().unwrap())
            };
            self.new_head += 1;
            return Some(result);
        }

        while self.old_head < self.old_edge {
            if let Some(old_head_id) = self.old_ids[self.old_head].take() {
                self.old_head += 1;
                return Some(ReconcileResult::Remove(old_head_id));
            } else {
                self.old_head += 1;
            }
        }

        debug_assert!(self.old_ids.iter().all(Option::is_none));
        debug_assert!(self.new_values.iter().all(Option::is_none));

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.old_keys.len().max(self.new_keys.len()),
            Some(self.old_keys.len() + self.old_keys.len()),
        )
    }
}

impl<Key, Id, Value> FusedIterator for Reconciler<Key, Id, Value>
where
    Id: Copy,
    Key: Eq + Hash + Copy,
{
}
