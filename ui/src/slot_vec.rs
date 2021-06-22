use std::convert::TryFrom;
use std::ops::{Index, IndexMut};
use std::fmt::{self, Debug, Formatter};

#[derive(Debug)]
pub struct SlotVec<T> {
    entries: Vec<(usize, T)>,
    slots: Vec<Slot>,
    free_indexes: Vec<usize>,
}

#[derive(Eq, PartialEq, Clone)]
struct Slot(isize);

impl<T> SlotVec<T> {
    pub const fn new() -> SlotVec<T> {
        SlotVec {
            entries: Vec::new(),
            slots: Vec::new(),
            free_indexes: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> SlotVec<T> {
        SlotVec {
            entries: Vec::with_capacity(capacity),
            slots: Vec::with_capacity(capacity),
            free_indexes: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> usize {
        let slot_index = if let Some(slot_index) = self.free_indexes.pop() {
            debug_assert!(self.slots[slot_index].is_free());
            self.slots[slot_index] = Slot::filled(self.entries.len());
            slot_index
        } else {
            let slot_index = self.slots.len();
            self.slots.push(Slot::filled(slot_index));
            slot_index
        };

        self.entries.push((slot_index, value));

        slot_index
    }

    pub fn remove(&mut self, slot_index: usize) -> T {
        let entry_index = self.slots[slot_index].as_filled()
            .expect("already removed entry");

        if slot_index == self.slots.len().saturating_sub(1) {
            self.slots.pop();
            self.truncate_to_fit();
        } else {
            self.slots[slot_index] = Slot::free(self.free_indexes.len());
            self.free_indexes.push(slot_index);
        }

        if entry_index == self.entries.len().saturating_sub(1) {
            self.entries.pop().unwrap().1
        } else {
            let swap_slot_index = self.entries[self.entries.len() - 1].0;
            self.slots[swap_slot_index] = Slot::filled(entry_index);
            self.entries.swap_remove(entry_index).1
        }
    }

    pub fn get(&self, slot_index: usize) -> Option<&T> {
        let entries = &self.entries;
        self.slots
            .get(slot_index)
            .and_then(move |slot| {
                if let Some(entry_index) = slot.as_filled() {
                    Some(&entries[entry_index].1)
                } else {
                    None
                }
            })
    }

    pub fn get_mut(&mut self, slot_index: usize) -> Option<&mut T> {
        let entries = &mut self.entries;
        self.slots
            .get(slot_index)
            .and_then(move |slot| {
                if let Some(entry_index) = slot.as_filled() {
                    Some(&mut entries[entry_index].1)
                } else {
                    None
                }
            })
    }

    pub fn has(&self, slot_index: usize) -> bool {
        self.slots
            .get(slot_index)
            .map_or(false, |slot| slot.is_filled())
    }

    pub fn clear(&mut self) {
        self.entries = Vec::new();
        self.slots = Vec::new();
        self.free_indexes = Vec::new();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn slot_size(&self) -> usize {
        self.slots.len()
    }

    #[inline]
    pub fn next_slot_index(&self) -> usize {
        self.free_indexes
            .last()
            .copied()
            .unwrap_or(self.slots.len())
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries
            .iter()
            .map(|entry| &entry.1)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries
            .iter_mut()
            .map(|entry| &mut entry.1)
    }

    pub fn entries(&self) -> impl Iterator<Item = (usize, &T)> {
        self.entries
            .iter()
            .map(|entry| (entry.0, &entry.1))
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.entries
            .iter_mut()
            .map(|entry| (entry.0, &mut entry.1))
    }

    fn truncate_to_fit(&mut self) {
        let mut removable_len = 0;

        for i in (0..self.slots.len()).rev() {
            let slot = &self.slots[i];
            if slot.is_filled() {
                break;
            }

            let index = slot.force_free();
            if index == self.free_indexes.len().saturating_sub(1) {
                self.free_indexes.pop();
            } else {
                let free_slot_index = self.free_indexes.swap_remove(index);
                debug_assert_eq!(free_slot_index, i);
                let slot_index = self.free_indexes[index];
                self.slots[slot_index] = slot.clone();
            }

            removable_len += 1;
        }

        if removable_len > 0 {
            self.slots.truncate(self.slots.len() - removable_len);
        }
    }
}

impl<T> Index<usize> for SlotVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        let entry_index = self.slots[index].as_filled()
            .expect("already removed entry");
        &self.entries[entry_index].1
    }
}

impl<T> IndexMut<usize> for SlotVec<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        let entry_index = self.slots[index].as_filled()
            .expect("already removed entry");
        &mut self.entries[entry_index].1
    }
}

impl Slot {
    fn filled(index: usize) -> Slot {
        Slot(isize::try_from(index).expect("overflow index"))
    }

    fn free(index: usize) -> Slot {
        Slot(-isize::try_from(index + 1).expect("overflow index"))
    }

    fn is_filled(&self) -> bool {
        self.0 >= 0
    }

    fn is_free(&self) -> bool {
        self.0 < 0
    }

    fn force_free(&self) -> usize {
        debug_assert!(self.is_free());
        -(self.0 + 1) as usize
    }

    fn force_filled(&self) -> usize {
        debug_assert!(self.is_filled());
        self.0 as usize
    }

    fn as_filled(&self) -> Option<usize> {
        if self.is_filled() {
            Some(self.force_filled())
        } else {
            None
        }
    }
}

impl Debug for Slot {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if self.is_filled() {
            formatter
                .debug_tuple("Slot::filled")
                .field(&self.force_filled())
                .finish()
        } else {
            formatter
                .debug_tuple("Slot::free")
                .field(&self.force_free())
                .finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_insert_and_remove() {
        let mut xs = SlotVec::new();

        assert_eq!(xs.next_slot_index(), 0);
        assert_eq!(xs.entries, []);
        assert_eq!(xs.slots, []);

        assert_eq!(xs.insert("foo"), 0);
        assert_eq!(xs.next_slot_index(), 1);
        assert_eq!(xs.entries, [(0, "foo")]);
        assert_eq!(xs.free_indexes, []);
        assert_eq!(xs.slots, [Slot::filled(0)]);

        assert_eq!(xs.insert("bar"), 1);
        assert_eq!(xs.next_slot_index(), 2);
        assert_eq!(xs.entries, [(0, "foo"), (1, "bar")]);
        assert_eq!(xs.free_indexes, []);
        assert_eq!(xs.slots, [Slot::filled(0), Slot::filled(1)]);

        assert_eq!(xs.insert("baz"), 2);
        assert_eq!(xs.next_slot_index(), 3);
        assert_eq!(xs.entries, [(0, "foo"), (1, "bar"), (2, "baz")]);
        assert_eq!(xs.free_indexes, []);
        assert_eq!(xs.slots, [Slot::filled(0), Slot::filled(1), Slot::filled(2)]);

        assert_eq!(xs.insert("qux"), 3);
        assert_eq!(xs.next_slot_index(), 4);
        assert_eq!(xs.entries, [(0, "foo"), (1, "bar"), (2, "baz"), (3, "qux")]);
        assert_eq!(xs.free_indexes, []);
        assert_eq!(xs.slots, [Slot::filled(0), Slot::filled(1), Slot::filled(2), Slot::filled(3)]);

        assert_eq!(xs.insert("quux"), 4);
        assert_eq!(xs.next_slot_index(), 5);
        assert_eq!(xs.entries, [(0, "foo"), (1, "bar"), (2, "baz"), (3, "qux"), (4, "quux")]);
        assert_eq!(xs.free_indexes, []);
        assert_eq!(xs.slots, [Slot::filled(0), Slot::filled(1), Slot::filled(2), Slot::filled(3), Slot::filled(4)]);

        assert_eq!(xs.remove(3), "qux");
        assert_eq!(xs.next_slot_index(), 3);
        assert_eq!(xs.entries, [(0, "foo"), (1, "bar"), (2, "baz"), (4, "quux")]);
        assert_eq!(xs.free_indexes, [3]);
        assert_eq!(xs.slots, [Slot::filled(0), Slot::filled(1), Slot::filled(2), Slot::free(0), Slot::filled(3)]);

        assert_eq!(xs.remove(1), "bar");
        assert_eq!(xs.next_slot_index(), 1);
        assert_eq!(xs.entries, [(0, "foo"), (4, "quux"), (2, "baz")]);
        assert_eq!(xs.free_indexes, [3, 1]);
        assert_eq!(xs.slots, [Slot::filled(0), Slot::free(1), Slot::filled(2), Slot::free(0), Slot::filled(1)]);

        assert_eq!(xs.remove(0), "foo");
        assert_eq!(xs.next_slot_index(), 0);
        assert_eq!(xs.entries, [(2, "baz"), (4, "quux")]);
        assert_eq!(xs.free_indexes, [3, 1, 0]);
        assert_eq!(xs.slots, [Slot::free(2), Slot::free(1), Slot::filled(0), Slot::free(0), Slot::filled(1)]);

        assert_eq!(xs.remove(4), "quux");
        assert_eq!(xs.next_slot_index(), 1);
        assert_eq!(xs.entries, [(2, "baz")]);
        assert_eq!(xs.free_indexes, [0, 1]);
        assert_eq!(xs.slots, [Slot::free(0), Slot::free(1), Slot::filled(0)]);

        assert_eq!(xs.insert("corge"), 1);
        assert_eq!(xs.next_slot_index(), 0);
        assert_eq!(xs.entries, [(2, "baz"), (1, "corge")]);
        assert_eq!(xs.free_indexes, [0]);
        assert_eq!(xs.slots, [Slot::free(0), Slot::filled(1), Slot::filled(0)]);

        assert_eq!(xs.remove(2), "baz");
        assert_eq!(xs.next_slot_index(), 0);
        assert_eq!(xs.entries, [(1, "corge")]);
        assert_eq!(xs.free_indexes, [0]);
        assert_eq!(xs.slots, [Slot::free(0), Slot::filled(0)]);

        assert_eq!(xs.remove(1), "corge");
        assert_eq!(xs.next_slot_index(), 0);
        assert_eq!(xs.entries, []);
        assert_eq!(xs.free_indexes, []);
        assert_eq!(xs.slots, []);
    }

    #[test]
    pub fn test_get() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.insert("baz");

        assert_eq!(xs.len(), 3);
        assert_eq!(xs.slot_size(), 3);

        assert_eq!(xs.get(foo), Some(&"foo"));
        assert_eq!(xs.get(bar), Some(&"bar"));
        assert_eq!(xs.get(baz), Some(&"baz"));
        assert_eq!(xs.get(baz + 1), None);

        assert_eq!(xs.get_mut(foo), Some(&mut "foo"));
        assert_eq!(xs.get_mut(bar), Some(&mut "bar"));
        assert_eq!(xs.get_mut(baz), Some(&mut "baz"));
        assert_eq!(xs.get_mut(baz + 1), None);

        assert_eq!(xs.has(foo), true);
        assert_eq!(xs.has(bar), true);
        assert_eq!(xs.has(baz), true);
        assert_eq!(xs.has(baz + 1), false);

        xs.remove(foo);
        xs.remove(bar);

        assert_eq!(xs.len(), 1);
        assert_eq!(xs.slot_size(), 3);

        assert_eq!(xs.get(foo), None);
        assert_eq!(xs.get(bar), None);
        assert_eq!(xs.get(baz), Some(&"baz"));
        assert_eq!(xs.get(baz + 1), None);

        assert_eq!(xs.get_mut(foo), None);
        assert_eq!(xs.get_mut(bar), None);
        assert_eq!(xs.get_mut(baz), Some(&mut "baz"));
        assert_eq!(xs.get_mut(baz + 1), None);

        assert_eq!(xs.has(foo), false);
        assert_eq!(xs.has(bar), false);
        assert_eq!(xs.has(baz), true);
        assert_eq!(xs.has(baz + 1), false);

        xs.clear();

        assert_eq!(xs.len(), 0);
        assert_eq!(xs.slot_size(), 0);

        assert_eq!(xs.get(foo), None);
        assert_eq!(xs.get(bar), None);
        assert_eq!(xs.get(baz), None);
        assert_eq!(xs.get(baz + 1), None);

        assert_eq!(xs.get_mut(foo), None);
        assert_eq!(xs.get_mut(bar), None);
        assert_eq!(xs.get_mut(baz), None);
        assert_eq!(xs.get_mut(baz + 1), None);

        assert_eq!(xs.has(foo), false);
        assert_eq!(xs.has(bar), false);
        assert_eq!(xs.has(baz), false);
        assert_eq!(xs.has(baz + 1), false);
    }

    #[test]
    pub fn test_iterator() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.insert("baz");

        assert_eq!(xs.iter().collect::<Vec<_>>(), [&"foo", &"bar", &"baz"]);
        assert_eq!(xs.iter_mut().collect::<Vec<_>>(), [&mut "foo", &mut "bar", &mut "baz"]);
        assert_eq!(xs.entries().collect::<Vec<_>>(), [(foo, &"foo"), (bar, &"bar"), (baz, &"baz")]);
        assert_eq!(xs.entries_mut().collect::<Vec<_>>(), [(foo, &mut "foo"), (bar, &mut "bar"), (baz, &mut "baz")]);

        xs.remove(foo);

        assert_eq!(xs.iter().collect::<Vec<_>>(), [&"baz", &"bar"]);
        assert_eq!(xs.iter_mut().collect::<Vec<_>>(), [&mut "baz", &mut "bar"]);
        assert_eq!(xs.entries().collect::<Vec<_>>(), [(baz, &"baz"), (bar, &"bar")]);
        assert_eq!(xs.entries_mut().collect::<Vec<_>>(), [(baz, &mut "baz"), (bar, &mut "bar")]);

        xs.remove(baz);

        assert_eq!(xs.iter().collect::<Vec<_>>(), [&"bar"]);
        assert_eq!(xs.iter_mut().collect::<Vec<_>>(), [&mut "bar"]);
        assert_eq!(xs.entries().collect::<Vec<_>>(), [(bar, &"bar")]);
        assert_eq!(xs.entries_mut().collect::<Vec<_>>(), [(bar, &mut "bar")]);

        xs.remove(bar);

        assert_eq!(xs.iter().collect::<Vec<_>>(), &[] as &[&&str]);
        assert_eq!(xs.iter_mut().collect::<Vec<_>>(), &[] as &[&mut &str]);
        assert_eq!(xs.entries().collect::<Vec<_>>(), &[] as &[(usize, &&str)]);
        assert_eq!(xs.entries_mut().collect::<Vec<_>>(), &[] as &[(usize, &mut &str)]);
    }
}
