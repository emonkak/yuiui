use std::any::{Any, TypeId};
use std::collections::{hash_set, HashSet};
use std::hash::Hash;
use std::iter::{ExactSizeIterator, FusedIterator};

use crate::id::IdPathBuf;

pub trait Event<'event> {
    fn collect_types(type_ids: &mut Vec<TypeId>);

    fn from_any(event: &'event dyn Any) -> Option<Self>
    where
        Self: Sized;
}

impl<'event> Event<'event> for () {
    fn collect_types(_type_ids: &mut Vec<TypeId>) {}

    fn from_any(_event: &'event dyn Any) -> Option<Self> {
        None
    }
}

impl<'event, T: 'static> Event<'event> for &'event T {
    fn collect_types(type_ids: &mut Vec<TypeId>) {
        type_ids.push(TypeId::of::<T>())
    }

    fn from_any(event: &'event dyn Any) -> Option<Self> {
        event.downcast_ref()
    }
}

pub trait EventListener<'event> {
    type Event: Event<'event>;
}

#[derive(Debug)]
pub enum Lifecycle<T> {
    Mount,
    Update(T),
    Unmount,
}

impl<T> Lifecycle<T> {
    pub fn map<F, U>(self, f: F) -> Lifecycle<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Mount => Lifecycle::Mount,
            Self::Update(value) => Lifecycle::Update(f(value)),
            Self::Unmount => Lifecycle::Unmount,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventDestination {
    Global,
    Local(IdPathBuf),
    Upward(IdPathBuf),
    Downward(IdPathBuf),
}

pub type EventMask = OptionHashSet<TypeId>;

#[derive(Debug)]
pub struct OptionHashSet<T> {
    entries: Option<HashSet<T>>,
}

impl<T> OptionHashSet<T>
where
    T: Eq + Hash,
{
    pub const fn new() -> Self {
        Self { entries: None }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.as_ref().map_or(false, HashSet::is_empty)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.as_ref().map_or(0, HashSet::len)
    }

    #[inline]
    pub fn contains(&self, value: &T) -> bool {
        self.entries
            .as_ref()
            .map_or(false, |entries| entries.contains(value))
    }

    #[inline]
    pub fn insert(&mut self, value: T) {
        self.entries.get_or_insert_with(HashSet::new).insert(value);
    }
}

impl<T> Extend<T> for OptionHashSet<T>
where
    T: Eq + Hash,
{
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.entries.get_or_insert_with(HashSet::new).extend(iter);
    }
}

impl<'a, T> Extend<&'a T> for OptionHashSet<T>
where
    T: Eq + Hash + Copy + 'a,
{
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        self.entries.get_or_insert_with(HashSet::new).extend(iter);
    }
}

impl<T> IntoIterator for OptionHashSet<T>
where
    T: Eq + Hash,
{
    type Item = T;

    type IntoIter = OptionIter<hash_set::IntoIter<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.entries.map(HashSet::into_iter);
        OptionIter::new(iter)
    }
}

impl<'a, T> IntoIterator for &'a OptionHashSet<T>
where
    T: Eq + Hash,
{
    type Item = &'a T;

    type IntoIter = OptionIter<hash_set::Iter<'a, T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.entries.as_ref().map(HashSet::iter);
        OptionIter::new(iter)
    }
}

pub struct OptionIter<I> {
    iter: Option<I>,
}

impl<T> OptionIter<T> {
    fn new(iter: Option<T>) -> Self {
        Self { iter }
    }
}

impl<I: Iterator> Iterator for OptionIter<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.as_mut().and_then(Iterator::next)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter
            .as_ref()
            .map_or_else(|| (0, Some(0)), Iterator::size_hint)
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for OptionIter<I> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.as_ref().map_or(0, ExactSizeIterator::len)
    }
}

impl<I: FusedIterator> FusedIterator for OptionIter<I> {}
