use std::fmt;
use std::mem;
use std::ops::{Index, IndexMut};
use std::vec;

pub type Key = usize;

#[derive(Clone, Default)]
pub struct SlotVec<T> {
    entries: Vec<(Key, T)>,
    slots: Vec<Slot>,
    free_indexes: Vec<usize>,
}

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

    #[inline]
    pub fn contains_key(&self, key: Key) -> bool {
        self.slots.get(key).map_or(false, |slot| slot.is_filled())
    }

    #[inline]
    pub fn get(&self, key: Key) -> Option<&T> {
        let entries = &self.entries;
        self.slots
            .get(key)
            .and_then(|slot| slot.as_filled())
            .map(move |index| {
                let (_, value) = &entries[index];
                value
            })
    }

    #[inline]
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        let entries = &mut self.entries;
        self.slots
            .get(key)
            .and_then(|slot| slot.as_filled())
            .map(move |index| {
                let (_, value) = &mut entries[index];
                value
            })
    }

    pub fn get_or_insert(&mut self, key: Key, value: T) -> &mut T {
        self.get_or_insert_with(key, || value)
    }

    pub fn get_or_insert_default(&mut self, key: Key) -> &mut T
    where
        T: Default,
    {
        self.get_or_insert_with(key, Default::default)
    }

    pub fn get_or_insert_with(&mut self, key: Key, f: impl FnOnce() -> T) -> &mut T {
        match self.slots.get(key) {
            Some(slot) if slot.is_filled() => {
                let index = slot.force_filled();
                let (_, value) = &mut self.entries[index];
                return value;
            }
            Some(slot) => {
                let free_position = slot.force_free();
                let free_index = self.consume_free_slot(free_position);
                debug_assert_eq!(free_index, key);
                self.slots[key] = Slot::filled(self.entries.len());
            }
            None => {
                self.extend_until(key);
                self.slots.push(Slot::filled(self.entries.len()));
            }
        }

        self.entries.push((key, f()));

        let (_, value) = self.entries.last_mut().unwrap();

        value
    }

    pub fn push(&mut self, value: T) -> Key {
        let key = if let Some(index) = self.free_indexes.pop() {
            debug_assert!(self.slots[index].is_free());
            self.slots[index] = Slot::filled(self.entries.len());
            index
        } else {
            let index = self.slots.len();
            self.slots.push(Slot::filled(self.entries.len()));
            index
        };

        self.entries.push((key, value));

        key
    }

    pub fn insert(&mut self, key: Key, value: T) -> Option<T> {
        match self.slots.get(key) {
            Some(slot) if slot.is_filled() => {
                let index = slot.force_filled();
                let (_, old_value) = &mut self.entries[index];
                return Some(mem::replace(old_value, value));
            }
            Some(slot) if slot.is_free() => {
                let free_position = slot.force_free();
                let free_index = self.consume_free_slot(free_position);
                debug_assert_eq!(free_index, key);
                self.slots[key] = Slot::filled(self.entries.len());
            }
            Some(_) => {
                self.slots[key] = Slot::filled(self.entries.len());
            }
            None => {
                self.extend_until(key);
                self.slots.push(Slot::filled(self.entries.len()));
            }
        }

        self.entries.push((key, value));

        None
    }

    pub fn reserve_key(&mut self) -> Key {
        let index = self.slots.len();
        self.slots.push(Slot::NULL);
        index
    }

    pub fn remove(&mut self, key: Key) -> Option<T> {
        if let Some(index) = self.slots.get(key).and_then(|slot| slot.as_filled()) {
            if key == self.slots.len().saturating_sub(1) {
                self.slots.pop();
                self.truncate_to_fit();
            } else {
                self.slots[key] = Slot::free(self.free_indexes.len());
                self.free_indexes.push(key);
            }

            Some(self.remove_entry(index))
        } else {
            None
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.slots.clear();
        self.free_indexes.clear();
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
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
    pub fn next_key(&self) -> Key {
        self.free_indexes
            .last()
            .copied()
            .unwrap_or(self.slots.len())
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.entries)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(&mut self.entries)
    }

    #[inline]
    pub fn ordered(&self) -> Ordered<'_, T> {
        Ordered::new(&self.entries, &self.slots)
    }

    #[inline]
    pub fn ordered_mut(&mut self) -> OrderedMut<'_, T> {
        OrderedMut::new(&mut self.entries, &mut self.slots)
    }

    fn remove_entry(&mut self, index: usize) -> T {
        if index == self.entries.len().saturating_sub(1) {
            let (_, value) = self.entries.pop().unwrap();
            value
        } else {
            let (key, _) = &self.entries[self.entries.len() - 1];
            self.slots[*key] = Slot::filled(index);
            let (_, value) = self.entries.swap_remove(index);
            value
        }
    }

    fn truncate_to_fit(&mut self) {
        let mut removable_len = 0;

        for index in (0..self.slots.len()).rev() {
            let slot = &self.slots[index];
            if !slot.is_free() {
                break;
            }

            let free_position = slot.force_free();
            let free_index = self.consume_free_slot(free_position);
            debug_assert_eq!(free_index, index);

            removable_len += 1;
        }

        if removable_len > 0 {
            self.slots.truncate(self.slots.len() - removable_len);
        }
    }

    fn extend_until(&mut self, index: usize) {
        for _ in self.slots.len()..index {
            let free_index = self.slots.len();
            self.slots.push(Slot::free(self.free_indexes.len()));
            self.free_indexes.push(free_index);
        }
        debug_assert_eq!(index, self.slots.len());
    }

    fn consume_free_slot(&mut self, position: usize) -> usize {
        if position == self.free_indexes.len().saturating_sub(1) {
            self.free_indexes.pop().unwrap()
        } else {
            let free_index = self.free_indexes.swap_remove(position);
            let new_index = self.free_indexes[position];
            self.slots[new_index] = Slot::free(position);
            free_index
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for SlotVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_map = f.debug_map();
        for (key, value) in self.ordered() {
            debug_map.entry(&key, value);
        }
        debug_map.finish()
    }
}

impl<T> Index<Key> for SlotVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, key: Key) -> &T {
        let index = self.slots[key]
            .as_filled()
            .unwrap_or_else(|| panic!("invalid key: {}", key));
        let (_, value) = &self.entries[index];
        value
    }
}

impl<T> IndexMut<Key> for SlotVec<T> {
    #[inline]
    fn index_mut(&mut self, key: Key) -> &mut T {
        let index = self.slots[key]
            .as_filled()
            .unwrap_or_else(|| panic!("invalid key: {}", key));
        let (_, value) = &mut self.entries[index];
        value
    }
}

impl<T> IntoIterator for SlotVec<T> {
    type IntoIter = vec::IntoIter<(Key, T)>;

    type Item = (Key, T);

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a SlotVec<T> {
    type IntoIter = Iter<'a, T>;

    type Item = (Key, &'a T);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SlotVec<T> {
    type IntoIter = IterMut<'a, T>;

    type Item = (Key, &'a mut T);

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct Iter<'a, T> {
    entries: &'a [(Key, T)],
    current: usize,
}

impl<'a, T> Iter<'a, T> {
    fn new(entries: &'a [(Key, T)]) -> Self {
        Self {
            entries,
            current: 0,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Key, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.entries.len() {
            let (key, value) = &self.entries[self.current];
            self.current += 1;
            Some((*key, value))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.entries.len();
        (size, Some(size))
    }
}

pub struct IterMut<'a, T> {
    entries: &'a mut [(Key, T)],
    current: usize,
}

impl<'a, T> IterMut<'a, T> {
    fn new(entries: &'a mut [(Key, T)]) -> Self {
        Self {
            entries,
            current: 0,
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Key, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.entries.len() {
            let (key, value) = unsafe { &mut *(&mut self.entries[self.current] as *mut (Key, T)) };
            self.current += 1;
            Some((*key, value))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.entries.len();
        (size, Some(size))
    }
}

pub struct Ordered<'a, T> {
    entries: &'a [(Key, T)],
    slots: &'a [Slot],
    current: usize,
}

impl<'a, T> Ordered<'a, T> {
    fn new(entries: &'a [(Key, T)], slots: &'a [Slot]) -> Self {
        Self {
            entries,
            slots,
            current: 0,
        }
    }
}

impl<'a, T> Iterator for Ordered<'a, T> {
    type Item = (Key, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.slots.len() {
            let current = self.current;
            self.current += 1;
            if let Some(index) = self.slots[current].as_filled() {
                let (key, value) = &self.entries[index];
                return Some((*key, value));
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.entries.len();
        (size, Some(size))
    }
}

pub struct OrderedMut<'a, T> {
    entries: &'a mut [(Key, T)],
    slots: &'a [Slot],
    current: usize,
}

impl<'a, T> OrderedMut<'a, T> {
    fn new(entries: &'a mut [(Key, T)], slots: &'a mut [Slot]) -> Self {
        Self {
            entries,
            slots,
            current: 0,
        }
    }
}

impl<'a, T> Iterator for OrderedMut<'a, T> {
    type Item = (Key, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.slots.len() {
            let current = self.current;
            self.current += 1;
            if let Some(index) = self.slots[current].as_filled() {
                let (key, value) = unsafe { &mut *(&mut self.entries[index] as *mut (Key, T)) };
                return Some((*key, value));
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.entries.len();
        (size, Some(size))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Slot(isize);

impl Slot {
    const NULL: Self = Self(0);

    fn filled(index: usize) -> Self {
        Self(isize::try_from(index + 1).expect("overflow index"))
    }

    fn free(index: usize) -> Self {
        Self(-isize::try_from(index + 1).expect("overflow index"))
    }

    fn is_filled(&self) -> bool {
        self.0 > 0
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
        (self.0 - 1) as usize
    }

    fn as_filled(&self) -> Option<usize> {
        if self.is_filled() {
            Some(self.force_filled())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use super::*;

    #[test]
    fn test_with_capacity() {
        let xs: SlotVec<&str> = SlotVec::with_capacity(100);
        assert_eq!(xs.capacity(), 100);
    }

    #[test]
    fn test_insert() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.next_key();

        assert_eq!(xs.contains_key(foo), true);
        assert_eq!(xs.contains_key(bar), true);
        assert_eq!(xs.contains_key(baz), false);

        assert_eq!(xs.get(foo), Some(&"foo"));
        assert_eq!(xs.get(bar), Some(&"bar"));
        assert_eq!(xs.get(baz), None);

        assert_eq!(xs.get_mut(foo), Some(&mut "foo"));
        assert_eq!(xs.get_mut(bar), Some(&mut "bar"));
        assert_eq!(xs.get_mut(baz), None);

        assert_eq!(&xs[foo], &"foo");
        assert_eq!(&xs[bar], &"bar");
        assert!(catch_unwind(|| &xs[baz]).is_err());

        assert_eq!(&mut xs[foo], &mut "foo");
        assert_eq!(&mut xs[bar], &mut "bar");
        assert!(catch_unwind(move || {
            xs.index_mut(baz);
        })
        .is_err());
    }

    #[test]
    fn test_reserve_key() {
        let mut xs = SlotVec::new();
        let null = xs.reserve_key();
        let foo = xs.push("foo");

        assert_ne!(foo, null);
        assert_eq!(xs.get(foo), Some(&"foo"));
        assert_eq!(xs.get(null), None);
        assert_eq!(xs[foo], "foo");
        assert!(catch_unwind(|| xs[null]).is_err());

        assert_eq!(xs.insert(null, "bar"), None);
        assert_eq!(xs.get(null), Some(&"bar"));
        assert_eq!(xs[null], "bar");
    }

    #[test]
    fn test_get_or_insert() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.next_key();

        assert_eq!(xs.get_or_insert(foo, "baz"), &mut "foo");
        assert_eq!(xs.get_or_insert(bar, "baz"), &mut "bar");
        assert_eq!(xs.get_or_insert(baz, "baz"), &mut "baz");
    }

    #[test]
    fn test_get_or_insert_default() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.next_key();

        assert_eq!(xs.get_or_insert_default(foo), &mut "foo");
        assert_eq!(xs.get_or_insert_default(bar), &mut "bar");
        assert_eq!(xs.get_or_insert_default(baz), &mut "");
    }

    #[test]
    fn test_get_or_insert_with() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.next_key();
        let f = || "baz";

        assert_eq!(xs.get_or_insert_with(foo, f), &mut "foo");
        assert_eq!(xs.get_or_insert_with(bar, f), &mut "bar");
        assert_eq!(xs.get_or_insert_with(baz, f), &mut "baz");
    }

    #[test]
    fn test_replace() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.next_key();

        assert_eq!(xs.insert(foo, "baz"), Some("foo"));
        assert_eq!(xs.insert(bar, "baz"), Some("bar"));
        assert_eq!(xs.insert(baz, "baz"), None);

        assert_eq!(xs[foo], "baz");
        assert_eq!(xs[bar], "baz");
        assert_eq!(xs[baz], "baz");
    }

    #[test]
    fn test_remove() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.next_key();

        assert_eq!(xs.remove(foo), Some("foo"));
        assert_eq!(xs.len(), 1);
        assert_eq!(xs.slot_size(), 2);
        assert_eq!(xs.contains_key(foo), false);
        assert_eq!(xs.contains_key(bar), true);
        assert_eq!(xs.contains_key(baz), false);

        assert_eq!(Some("bar"), xs.remove(bar));
        assert_eq!(0, xs.len());
        assert_eq!(0, xs.slot_size());
        assert_eq!(false, xs.contains_key(foo));
        assert_eq!(false, xs.contains_key(bar));
        assert_eq!(false, xs.contains_key(baz));

        assert_eq!(None, xs.remove(baz));
    }

    #[test]
    fn test_clear() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.push("baz");

        assert_eq!(xs.len(), 3);
        assert_eq!(xs.slot_size(), 3);

        xs.clear();

        assert_eq!(xs.len(), 0);
        assert_eq!(xs.slot_size(), 0);
        assert_eq!(xs.contains_key(foo), false);
        assert_eq!(xs.contains_key(bar), false);
        assert_eq!(xs.contains_key(baz), false);
    }

    #[test]
    fn test_into_iter() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.push("baz");

        assert_eq!(
            xs.into_iter().collect::<Vec<_>>(),
            vec![(foo, "foo"), (bar, "bar"), (baz, "baz")]
        );
    }

    #[test]
    fn test_iter() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.push("baz");

        xs.remove(foo);

        assert_eq!(
            xs.iter().collect::<Vec<_>>(),
            vec![(baz, &"baz"), (bar, &"bar")],
        );
        assert_eq!(
            xs.iter_mut().collect::<Vec<_>>(),
            vec![(baz, &mut "baz"), (bar, &mut "bar")]
        );
    }

    #[test]
    fn test_ordered() {
        let mut xs = SlotVec::new();
        let foo = xs.push("foo");
        let bar = xs.push("bar");
        let baz = xs.push("baz");

        xs.remove(foo);

        assert_eq!(
            xs.ordered().collect::<Vec<_>>(),
            vec![(bar, &"bar"), (baz, &"baz")]
        );
        assert_eq!(
            xs.ordered_mut().collect::<Vec<_>>(),
            vec![(bar, &mut "bar"), (baz, &mut "baz")]
        );
    }
}
