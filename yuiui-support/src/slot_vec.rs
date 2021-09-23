use std::convert::TryFrom;
use std::fmt;
use std::mem;
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug, Default)]
pub struct SlotVec<T> {
    entries: Vec<(usize, T)>,
    slots: Vec<Slot>,
    free_indexes: Vec<usize>,
}

#[derive(Eq, PartialEq, Clone, Copy)]
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
            self.slots.push(Slot::filled(self.entries.len()));
            slot_index
        };

        self.entries.push((slot_index, value));

        slot_index
    }

    pub fn insert_null(&mut self) -> usize {
        let slot_index = self.slots.len();
        self.slots.push(Slot::NULL);
        slot_index
    }

    pub fn insert_at(&mut self, slot_index: usize, value: T) -> Option<T> {
        if let Some(slot) = self.slots.get(slot_index) {
            if slot.is_filled() {
                let entry_index = slot.force_filled();
                let old_value = mem::replace(&mut self.entries[entry_index].1, value);
                return Some(old_value);
            } else if slot.is_free() {
                let free_position = slot.force_free();
                let free_slot_index = self.take_free_index(free_position);
                debug_assert_eq!(free_slot_index, slot_index);

                self.slots[slot_index] = Slot::filled(self.entries.len());
            } else {
                unreachable!("null slot")
            }
        } else {
            self.extend_until(slot_index);
            self.slots.push(Slot::filled(self.entries.len()));
        }

        self.entries.push((slot_index, value));

        None
    }

    pub fn try_remove(&mut self, slot_index: usize) -> Option<T> {
        if let Some(entry_index) = self.slots[slot_index].as_filled() {
            if slot_index == self.slots.len().saturating_sub(1) {
                self.slots.pop();
                self.truncate_to_fit();
            } else {
                self.slots[slot_index] = Slot::free(self.free_indexes.len());
                self.free_indexes.push(slot_index);
            }

            Some(self.remove_entry(entry_index))
        } else {
            None
        }
    }

    pub fn remove(&mut self, slot_index: usize) -> T {
        self.try_remove(slot_index)
            .unwrap_or_else(|| panic!("Already removed entry at {}", slot_index))
    }

    pub fn get(&self, slot_index: usize) -> Option<&T> {
        let entries = &self.entries;
        self.slots.get(slot_index).and_then(move |slot| {
            slot.as_filled()
                .map(move |entry_index| &entries[entry_index].1)
        })
    }

    pub fn get_mut(&mut self, slot_index: usize) -> Option<&mut T> {
        let entries = &mut self.entries;
        self.slots.get(slot_index).and_then(move |slot| {
            slot.as_filled()
                .map(move |entry_index| &mut entries[entry_index].1)
        })
    }

    pub fn get_or_insert(&mut self, slot_index: usize, value: T) -> &mut T {
        self.get_or_insert_with(slot_index, || value)
    }

    pub fn get_or_insert_default(&mut self, slot_index: usize) -> &mut T
    where
        T: Default,
    {
        self.get_or_insert_with(slot_index, Default::default)
    }

    pub fn get_or_insert_with(&mut self, slot_index: usize, f: impl FnOnce() -> T) -> &mut T {
        if let Some(slot) = self.slots.get(slot_index) {
            if slot.is_filled() {
                let entry_index = slot.force_filled();
                return &mut self.entries[entry_index].1;
            }

            let free_position = slot.force_free();
            let free_slot_index = self.take_free_index(free_position);
            debug_assert_eq!(free_slot_index, slot_index);

            self.slots[slot_index] = Slot::filled(self.entries.len());
        } else {
            self.extend_until(slot_index);
            self.slots.push(Slot::filled(self.entries.len()));
        }

        self.entries.push((slot_index, f()));

        &mut self.entries.last_mut().unwrap().1
    }

    pub fn contains(&self, slot_index: usize) -> bool {
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
    pub fn next_slot_index(&self) -> usize {
        self.free_indexes
            .last()
            .copied()
            .unwrap_or(self.slots.len())
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().map(|entry| &entry.1)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries.iter_mut().map(|entry| &mut entry.1)
    }

    pub fn entries(&self) -> impl Iterator<Item = (usize, &T)> {
        self.entries.iter().map(|entry| (entry.0, &entry.1))
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.entries.iter_mut().map(|entry| (entry.0, &mut entry.1))
    }

    fn remove_entry(&mut self, entry_index: usize) -> T {
        if entry_index == self.entries.len().saturating_sub(1) {
            self.entries.pop().unwrap().1
        } else {
            let swap_slot_index = self.entries[self.entries.len() - 1].0;
            self.slots[swap_slot_index] = Slot::filled(entry_index);
            self.entries.swap_remove(entry_index).1
        }
    }

    fn truncate_to_fit(&mut self) {
        let mut removable_len = 0;

        for slot_index in (0..self.slots.len()).rev() {
            let slot = &self.slots[slot_index];
            if slot.is_filled() {
                break;
            }

            let free_position = slot.force_free();
            let free_slot_index = self.take_free_index(free_position);
            debug_assert_eq!(free_slot_index, slot_index);

            removable_len += 1;
        }

        if removable_len > 0 {
            self.slots.truncate(self.slots.len() - removable_len);
        }
    }

    fn extend_until(&mut self, slot_index: usize) {
        for _ in self.slots.len()..slot_index {
            let slot_index = self.slots.len();
            self.slots.push(Slot::free(self.free_indexes.len()));
            self.free_indexes.push(slot_index);
        }
        debug_assert_eq!(slot_index, self.slots.len());
    }

    fn take_free_index(&mut self, position: usize) -> usize {
        if position == self.free_indexes.len().saturating_sub(1) {
            self.free_indexes.pop().unwrap()
        } else {
            let free_slot_index = self.free_indexes.swap_remove(position);
            let swap_slot_index = self.free_indexes[position];
            self.slots[swap_slot_index] = Slot::free(position);
            free_slot_index
        }
    }
}

impl<T> Index<usize> for SlotVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        let entry_index = self.slots[index]
            .as_filled()
            .unwrap_or_else(|| panic!("Already removed entry at {}", index));
        &self.entries[entry_index].1
    }
}

impl<T> IndexMut<usize> for SlotVec<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        let entry_index = self.slots[index]
            .as_filled()
            .unwrap_or_else(|| panic!("Already removed entry at {}", index));
        &mut self.entries[entry_index].1
    }
}

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

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_filled() {
            f.debug_tuple("Slot::filled")
                .field(&self.force_filled())
                .finish()
        } else if self.is_free() {
            f.debug_tuple("Slot::free")
                .field(&self.force_free())
                .finish()
        } else {
            f.write_str("Slot::null")
        }
    }
}
