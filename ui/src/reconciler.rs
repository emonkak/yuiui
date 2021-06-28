use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub fn reconcile<Key, OldValue, NewValue>(
    commiter: &mut impl Commiter<OldValue, NewValue>,
    target_value: OldValue,
    old_keys: &[Key],
    old_values: &mut [Option<OldValue>],
    new_keys: &[Key],
    new_values: &mut [Option<NewValue>]
) where Key: Eq + Hash, OldValue: Clone + Copy + Default {
    let mut old_head = 0;
    let mut old_edge = old_keys.len();
    let mut new_head = 0;
    let mut new_edge = new_keys.len();

    let mut new_index_to_old_value: Vec<OldValue> = vec![Default::default(); new_keys.len()];
    let mut old_key_to_index_map: Option<HashMap<&Key, usize>> = None;
    let mut new_key_set: Option<HashSet<&Key>> = None;

    while old_head < old_edge && new_head < new_edge {
        let old_tail = old_edge - 1;
        let new_tail = new_edge - 1;

        match (old_values[old_head], old_values[old_tail]) {
            (None, _) => {
                old_head += 1;
            }
            (_, None) => {
                old_edge -= 1;
            },
            (Some(old_head_value), _) if old_keys[old_head] == new_keys[new_head] => {
                commiter.commit_update(old_head_value, new_values[new_head].take().unwrap());
                new_index_to_old_value[new_head] = old_head_value;
                old_head += 1;
                new_head += 1;
            }
            (_, Some(old_tail_value)) if old_keys[old_tail] == new_keys[new_tail] => {
                commiter.commit_update(old_tail_value, new_values[new_tail].take().unwrap());
                new_index_to_old_value[new_tail] = old_tail_value;
                old_head += 1;
                old_edge -= 1;
                new_edge -= 1;
            }
            (Some(old_head_value), Some(old_tail_value)) if old_keys[old_head] == new_keys[new_tail] => {
                commiter.commit_update_and_move(old_head_value, old_tail_value, new_values[new_tail].take().unwrap());
                new_index_to_old_value[new_tail] = old_head_value;
                old_head += 1;
                new_edge -= 1;
            }
            (Some(old_head_value), Some(old_tail_value)) if old_keys[old_tail] == new_keys[new_head] => {
                commiter.commit_update_and_move(old_tail_value, old_head_value, new_values[new_head].take().unwrap());
                new_index_to_old_value[new_head] = old_tail_value;
                old_edge -= 1;
                new_head += 1;
            }
            (Some(old_head_value), Some(old_tail_value)) => {
                let new_key_set = new_key_set.get_or_insert_with(|| {
                    new_keys.iter().collect::<HashSet<_>>()
                });

                if !new_key_set.contains(&old_keys[old_head]) {
                    commiter.commit_delete(old_head_value);
                    old_head += 1;
                } else if !new_key_set.contains(&old_keys[old_tail]) {
                    commiter.commit_delete(old_tail_value);
                    old_edge -= 1;
                } else {
                    let old_key_to_index_map = old_key_to_index_map.get_or_insert_with(|| {
                        let mut map = HashMap::with_capacity(old_keys.len());
                        for (i, key) in old_keys.iter().enumerate() {
                            map.insert(key, i);
                        }
                        map
                    });

                    if let Some(old_value) = old_key_to_index_map
                        .get(&new_keys[new_head])
                        .and_then(|&old_index| old_values[old_index].take()) {
                        commiter.commit_update_and_move(old_value, old_head_value, new_values[new_tail].take().unwrap());
                        new_index_to_old_value[new_tail] = old_value;
                    } else {
                        commiter.commit_place_at(old_head_value, new_values[new_head].take().unwrap());
                    }

                    new_head += 1;
                }
            }
        }
    }

    while new_head < new_edge {
        if new_edge < new_values.len() {
            let old_value = new_index_to_old_value[new_edge];
            commiter.commit_place_at(old_value, new_values[new_head].take().unwrap());
        } else {
            commiter.commit_place(target_value, new_values[new_head].take().unwrap());
        }
        new_head += 1;
    }

    while old_head < old_edge {
        if let Some(old_value) = old_values[old_head].take() {
            commiter.commit_delete(old_value);
        }
        old_head += 1;
    }

    debug_assert!(new_values.iter().all(Option::is_none));
}

pub trait Commiter<OldValue, NewValue> {
    fn commit_place(&mut self, ref_value: OldValue, new_value: NewValue);

    fn commit_place_at(&mut self, ref_value: OldValue, new_value: NewValue);

    fn commit_update(&mut self, target_value: OldValue, new_value: NewValue);

    fn commit_update_and_move(&mut self, target_value: OldValue, ref_value: OldValue, new_value: NewValue);

    fn commit_delete(&mut self, target_value: OldValue);
}
