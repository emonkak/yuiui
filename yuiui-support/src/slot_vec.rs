use std::fmt;
use std::mem;
use std::ops::{Index, IndexMut};

#[derive(Clone, Default)]
pub struct SlotVec<T> {
    entries: Vec<(usize, T)>,
    slots: Vec<Slot>,
    free_keys: Vec<usize>,
}

impl<T> SlotVec<T> {
    pub const fn new() -> SlotVec<T> {
        SlotVec {
            entries: Vec::new(),
            slots: Vec::new(),
            free_keys: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> SlotVec<T> {
        SlotVec {
            entries: Vec::with_capacity(capacity),
            slots: Vec::with_capacity(capacity),
            free_keys: Vec::new(),
        }
    }

    pub fn contains(&self, key: usize) -> bool {
        self.slots.get(key).map_or(false, |slot| slot.is_filled())
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        let entries = &self.entries;
        self.slots
            .get(key)
            .and_then(|slot| slot.as_filled())
            .map(move |index| &entries[index].1)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        let entries = &mut self.entries;
        self.slots
            .get(key)
            .and_then(|slot| slot.as_filled())
            .map(move |index| &mut entries[index].1)
    }

    pub fn get_or_insert(&mut self, key: usize, value: T) -> &mut T {
        self.get_or_insert_with(key, || value)
    }

    pub fn get_or_insert_default(&mut self, key: usize) -> &mut T
    where
        T: Default,
    {
        self.get_or_insert_with(key, Default::default)
    }

    pub fn get_or_insert_with(&mut self, key: usize, f: impl FnOnce() -> T) -> &mut T {
        if let Some(slot) = self.slots.get(key) {
            if slot.is_filled() {
                let index = slot.force_filled();
                return &mut self.entries[index].1;
            }

            let free_position = slot.force_free();
            let free_key = self.consume_free_slot(free_position);
            debug_assert_eq!(free_key, key);

            self.slots[key] = Slot::filled(self.entries.len());
        } else {
            self.extend_until(key);
            self.slots.push(Slot::filled(self.entries.len()));
        }

        self.entries.push((key, f()));

        &mut self.entries.last_mut().unwrap().1
    }

    pub fn insert(&mut self, value: T) -> usize {
        let key = if let Some(key) = self.free_keys.pop() {
            debug_assert!(self.slots[key].is_free());
            self.slots[key] = Slot::filled(self.entries.len());
            key
        } else {
            let key = self.slots.len();
            self.slots.push(Slot::filled(self.entries.len()));
            key
        };

        self.entries.push((key, value));

        key
    }

    pub fn insert_null(&mut self) -> usize {
        let key = self.slots.len();
        self.slots.push(Slot::NULL);
        key
    }

    pub fn replace(&mut self, key: usize, value: T) -> Option<T> {
        if let Some(slot) = self.slots.get(key) {
            if slot.is_filled() {
                let index = slot.force_filled();
                let old_value = mem::replace(&mut self.entries[index].1, value);
                return Some(old_value);
            } else if slot.is_free() {
                let free_position = slot.force_free();
                let free_key = self.consume_free_slot(free_position);
                debug_assert_eq!(free_key, key);
                self.slots[key] = Slot::filled(self.entries.len());
            } else {
                panic!("null slot")
            }
        } else {
            self.extend_until(key);
            self.slots.push(Slot::filled(self.entries.len()));
        }

        self.entries.push((key, value));

        None
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        if let Some(index) = self.slots.get(key).and_then(|slot| slot.as_filled()) {
            if key == self.slots.len().saturating_sub(1) {
                self.slots.pop();
                self.truncate_to_fit();
            } else {
                self.slots[key] = Slot::free(self.free_keys.len());
                self.free_keys.push(key);
            }

            Some(self.remove_entry(index))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.entries = Vec::new();
        self.slots = Vec::new();
        self.free_keys = Vec::new();
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
    pub fn next_key(&self) -> usize {
        self.free_keys.last().copied().unwrap_or(self.slots.len())
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.entries.iter().map(|entry| (entry.0, &entry.1))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.entries.iter_mut().map(|entry| (entry.0, &mut entry.1))
    }

    pub fn ordered(&self) -> impl Iterator<Item = (usize, &T)> {
        let entries = &self.entries;
        self.slots.iter().filter_map(|slot| {
            slot.as_filled().map(|index| {
                let (key, value) = &entries[index];
                (*key, value)
            })
        })
    }

    pub fn ordered_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        let entries: *mut _ = &mut self.entries;
        self.slots.iter().filter_map(move |slot| {
            slot.as_filled().map(|index| {
                let (key, value) = unsafe { &mut (*entries)[index] };
                (*key, value)
            })
        })
    }

    fn remove_entry(&mut self, index: usize) -> T {
        if index == self.entries.len().saturating_sub(1) {
            self.entries.pop().unwrap().1
        } else {
            let swap_key = self.entries[self.entries.len() - 1].0;
            self.slots[swap_key] = Slot::filled(index);
            self.entries.swap_remove(index).1
        }
    }

    fn truncate_to_fit(&mut self) {
        let mut removable_len = 0;

        for key in (0..self.slots.len()).rev() {
            let slot = &self.slots[key];
            if slot.is_filled() {
                break;
            }

            let free_position = slot.force_free();
            let free_key = self.consume_free_slot(free_position);
            debug_assert_eq!(free_key, key);

            removable_len += 1;
        }

        if removable_len > 0 {
            self.slots.truncate(self.slots.len() - removable_len);
        }
    }

    fn extend_until(&mut self, key: usize) {
        for _ in self.slots.len()..key {
            let key = self.slots.len();
            self.slots.push(Slot::free(self.free_keys.len()));
            self.free_keys.push(key);
        }
        debug_assert_eq!(key, self.slots.len());
    }

    fn consume_free_slot(&mut self, position: usize) -> usize {
        if position == self.free_keys.len().saturating_sub(1) {
            self.free_keys.pop().unwrap()
        } else {
            let free_key = self.free_keys.swap_remove(position);
            let alt_key = self.free_keys[position];
            self.slots[alt_key] = Slot::free(position);
            free_key
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

impl<T> Index<usize> for SlotVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, key: usize) -> &T {
        let index = self.slots[key]
            .as_filled()
            .unwrap_or_else(|| panic!("invalid key: {}", key));
        &self.entries[index].1
    }
}

impl<T> IndexMut<usize> for SlotVec<T> {
    #[inline]
    fn index_mut(&mut self, key: usize) -> &mut T {
        let index = self.slots[key]
            .as_filled()
            .unwrap_or_else(|| panic!("invalid key: {}", key));
        &mut self.entries[index].1
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
    use super::*;
    use std::panic::catch_unwind;

    #[test]
    fn test_with_capacity() {
        let xs: SlotVec<&str> = SlotVec::with_capacity(100);
        assert_eq!(100, xs.capacity());
    }

    #[test]
    fn test_contains() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.next_key();

        assert_eq!(true, xs.contains(foo));
        assert_eq!(true, xs.contains(bar));
        assert_eq!(false, xs.contains(baz));
    }

    #[test]
    fn test_index() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.next_key();

        assert_eq!(&"foo", xs.index(foo));
        assert_eq!(&"bar", xs.index(bar));
        assert!(catch_unwind(|| xs.index(baz)).is_err());

        assert_eq!(&mut "foo", xs.index_mut(foo));
        assert_eq!(&mut "bar", xs.index_mut(bar));
        assert!(catch_unwind(move || {
            xs.index_mut(baz);
        })
        .is_err());
    }

    #[test]
    fn test_get() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.next_key();

        assert_eq!(Some(&"foo"), xs.get(foo));
        assert_eq!(Some(&"bar"), xs.get(bar));
        assert_eq!(None, xs.get(baz));

        assert_eq!(Some(&mut "foo"), xs.get_mut(foo));
        assert_eq!(Some(&mut "bar"), xs.get_mut(bar));
        assert_eq!(None, xs.get_mut(baz));
    }

    #[test]
    fn test_get_or_insert() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.next_key();

        assert_eq!(&mut "foo", xs.get_or_insert(foo, "baz"));
        assert_eq!(&mut "bar", xs.get_or_insert(bar, "baz"));
        assert_eq!(&mut "baz", xs.get_or_insert(baz, "baz"));
    }

    #[test]
    fn test_get_or_insert_default() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.next_key();

        assert_eq!(&mut "foo", xs.get_or_insert_default(foo));
        assert_eq!(&mut "bar", xs.get_or_insert_default(bar));
        assert_eq!(&mut "", xs.get_or_insert_default(baz));
    }

    #[test]
    fn test_get_or_insert_with() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.next_key();
        let f = || "baz";

        assert_eq!(&mut "foo", xs.get_or_insert_with(foo, f));
        assert_eq!(&mut "bar", xs.get_or_insert_with(bar, f));
        assert_eq!(&mut "baz", xs.get_or_insert_with(baz, f));
    }

    #[test]
    fn test_insert_null() {
        let mut xs = SlotVec::new();
        let null = xs.insert_null();
        let foo = xs.insert("foo");

        assert_ne!(null, foo);
        assert_eq!("foo", xs[foo]);
        assert!(catch_unwind(|| xs[null]).is_err());
        assert!(catch_unwind(move || {
            xs.replace(null, "bar");
        })
        .is_err());
    }

    #[test]
    fn test_replace() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.next_key();

        assert_eq!(Some("foo"), xs.replace(foo, "baz"));
        assert_eq!(Some("bar"), xs.replace(bar, "baz"));
        assert_eq!(None, xs.replace(baz, "baz"));

        assert_eq!("baz", xs[foo]);
        assert_eq!("baz", xs[bar]);
        assert_eq!("baz", xs[baz]);
    }

    #[test]
    fn test_remove() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.next_key();

        assert_eq!(Some("foo"), xs.remove(foo));
        assert_eq!(1, xs.len());
        assert_eq!(2, xs.slot_size());
        assert_eq!(false, xs.contains(foo));
        assert_eq!(true, xs.contains(bar));
        assert_eq!(false, xs.contains(baz));

        assert_eq!(Some("bar"), xs.remove(bar));
        assert_eq!(0, xs.len());
        assert_eq!(0, xs.slot_size());
        assert_eq!(false, xs.contains(foo));
        assert_eq!(false, xs.contains(bar));
        assert_eq!(false, xs.contains(baz));

        assert_eq!(None, xs.remove(baz));
    }

    #[test]
    fn test_clear() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.insert("baz");

        assert_eq!(3, xs.len());
        assert_eq!(3, xs.slot_size());

        xs.clear();

        assert_eq!(0, xs.len());
        assert_eq!(0, xs.slot_size());
        assert_eq!(false, xs.contains(foo));
        assert_eq!(false, xs.contains(bar));
        assert_eq!(false, xs.contains(baz));
    }

    #[test]
    fn test_iter() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.insert("baz");

        assert_eq!(
            vec![(foo, &"foo"), (bar, &"bar"), (baz, &"baz")],
            xs.iter().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![(foo, &mut "foo"), (bar, &mut "bar"), (baz, &mut "baz")],
            xs.iter_mut().collect::<Vec<_>>()
        );

        assert_eq!(Some("foo"), xs.remove(foo));
        assert_eq!(
            vec![(baz, &"baz"), (bar, &"bar")],
            xs.iter().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![(baz, &mut "baz"), (bar, &mut "bar")],
            xs.iter_mut().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_ordered() {
        let mut xs = SlotVec::new();
        let foo = xs.insert("foo");
        let bar = xs.insert("bar");
        let baz = xs.insert("baz");

        assert_eq!(
            vec![(foo, &"foo"), (bar, &"bar"), (baz, &"baz")],
            xs.ordered().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![(foo, &mut "foo"), (bar, &mut "bar"), (baz, &mut "baz")],
            xs.ordered_mut().collect::<Vec<_>>()
        );

        assert_eq!(Some("foo"), xs.remove(foo));
        assert_eq!(
            vec![(bar, &"bar"), (baz, &"baz")],
            xs.ordered().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![(bar, &mut "bar"), (baz, &mut "baz")],
            xs.ordered_mut().collect::<Vec<_>>()
        );
    }
}
