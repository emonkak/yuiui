use std::cmp::Ordering;
use std::collections::VecDeque;

use crate::element::ElementSeq;
use crate::id::{Id, IdContext};
use crate::store::Store;
use crate::view_node::{CommitMode, Traversable, ViewNodeSeq};

use super::binary_search_by;

#[derive(Debug)]
pub struct VecStorage<T> {
    active: Vec<T>,
    staging: VecDeque<T>,
    new_len: usize,
    dirty: bool,
}

impl<T> VecStorage<T> {
    fn new(active: Vec<T>) -> Self {
        Self {
            staging: VecDeque::with_capacity(active.len()),
            new_len: active.len(),
            active,
            dirty: true,
        }
    }
}

impl<T, S, M, E> ElementSeq<S, M, E> for Vec<T>
where
    T: ElementSeq<S, M, E>,
{
    type Storage = VecStorage<T::Storage>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
        VecStorage::new(
            self.into_iter()
                .map(|element| element.render_children(id_context, state))
                .collect(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        let mut has_changed = storage.active.len() != self.len();

        storage
            .staging
            .reserve_exact(self.len().saturating_sub(storage.active.len()));
        storage.new_len = self.len();

        for (i, element) in self.into_iter().enumerate() {
            if i < storage.active.len() {
                let node = &mut storage.active[i];
                has_changed |= element.update_children(node, id_context, state);
            } else {
                let j = i - storage.active.len();
                if j < storage.staging.len() {
                    let node = &mut storage.staging[j];
                    has_changed |= element.update_children(node, id_context, state);
                } else {
                    let node = element.render_children(id_context, state);
                    storage.staging.push_back(node);
                    has_changed = true;
                }
            }
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<T, S, M, E> ViewNodeSeq<S, M, E> for VecStorage<T>
where
    T: ViewNodeSeq<S, M, E>,
{
    const SIZE_HINT: (usize, Option<usize>) = (0, None);

    fn len(&self) -> usize {
        match T::SIZE_HINT {
            (lower, Some(upper)) if lower == upper => lower * self.active.len(),
            _ => self.active.iter().map(|node| node.len()).sum(),
        }
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        if self.active.len() > 0 {
            let first = self.active[0].id_range();
            let last = self.active[self.active.len() - 1].id_range();
            match (first, last) {
                (Some((start, _)), Some((_, end))) => Some((start, end)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool {
        let mut result = false;
        if self.dirty || mode.is_propagable() {
            match self.new_len.cmp(&self.active.len()) {
                Ordering::Equal => {
                    // new_len == active_len
                    for node in &mut self.active {
                        result |= node.commit(mode, id_context, store, messages, entry_point);
                    }
                }
                Ordering::Less => {
                    // new_len < active_len
                    for node in &mut self.active[..self.new_len] {
                        result |= node.commit(mode, id_context, store, messages, entry_point);
                    }
                    for mut node in self.active.drain(self.new_len..).rev() {
                        result |= node.commit(
                            CommitMode::Unmount,
                            id_context,
                            store,
                            messages,
                            entry_point,
                        );
                        self.staging.push_front(node);
                    }
                }
                Ordering::Greater => {
                    // new_len > active_len
                    for node in &mut self.active {
                        result |= node.commit(mode, id_context, store, messages, entry_point);
                    }
                    if mode != CommitMode::Unmount {
                        for _ in 0..self.new_len - self.active.len() {
                            let mut node = self.staging.pop_front().unwrap();
                            result |= node.commit(
                                CommitMode::Mount,
                                id_context,
                                store,
                                messages,
                                entry_point,
                            );
                            self.active.push(node);
                        }
                    }
                }
            }
            self.dirty = false;
        }
        result
    }

    fn gc(&mut self) {
        if self.new_len <= self.active.len() {
            self.staging.clear();
        } else {
            let additional_len = self.new_len - self.active.len();
            self.staging.truncate(additional_len);
        }
        if !T::IS_STATIC {
            for node in &mut self.active {
                node.gc();
            }
            for node in &mut self.staging {
                node.gc();
            }
        }
    }
}

impl<T, Visitor, Context, S, M, E> Traversable<Visitor, Context, S, M, E> for VecStorage<T>
where
    T: Traversable<Visitor, Context, S, M, E> + ViewNodeSeq<S, M, E>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    ) {
        for node in &mut self.active {
            node.for_each(visitor, context, id_context);
        }
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    ) -> bool {
        if T::SIZE_HINT.1.is_some() {
            if let Ok(index) = binary_search_by(&self.active, |node| {
                node.id_range().map(|(start, end)| {
                    if start > id {
                        Ordering::Less
                    } else if end < id {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                })
            }) {
                let node = &mut self.active[index];
                return node.for_id(id, visitor, context, id_context);
            }
        } else {
            for node in &mut self.active {
                if node.for_id(id, visitor, context, id_context) {
                    return true;
                }
            }
        }
        false
    }
}
