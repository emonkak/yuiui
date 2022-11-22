use std::collections::VecDeque;

use crate::component_stack::ComponentStack;
use crate::element::{Element, ElementSeq};
use crate::id::{Id, IdStack};
use crate::store::Store;
use crate::view::View;
use crate::view_node::{CommitMode, Traversable, ViewNode, ViewNodeSeq};

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

impl<Element, S, M, E> ElementSeq<S, M, E> for Vec<Element>
where
    Element: self::Element<S, M, E>,
{
    type Storage = VecStorage<ViewNode<Element::View, Element::Components, S, M, E>>;

    fn render_children(self, id_stack: &mut IdStack, state: &S) -> Self::Storage {
        VecStorage::new(
            self.into_iter()
                .map(|element| element.render(id_stack, state))
                .collect(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_stack: &mut IdStack,
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
                has_changed |= element.update(node.into(), id_stack, state);
            } else {
                let j = i - storage.active.len();
                if j < storage.staging.len() {
                    let node = &mut storage.staging[j];
                    has_changed |= element.update(node.into(), id_stack, state);
                } else {
                    let node = element.render(id_stack, state);
                    storage.staging.push_back(node);
                    has_changed = true;
                }
            }
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<V, CS, S, M, E> ViewNodeSeq<S, M, E> for VecStorage<ViewNode<V, CS, S, M, E>>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    const SIZE_HINT: (usize, Option<usize>) = (0, None);

    fn len(&self) -> usize {
        self.active.len()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        id_stack: &mut IdStack,
        store: &Store<S>,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool {
        let mut result = false;
        if self.dirty || mode.is_propagable() {
            if self.new_len < self.active.len() {
                for node in &mut self.active[..self.new_len] {
                    result |= node.commit(mode, id_stack, store, messages, entry_point);
                }
                for mut node in self.active.drain(self.new_len..).rev() {
                    result |=
                        node.commit(CommitMode::Unmount, id_stack, store, messages, entry_point);
                    self.staging.push_front(node);
                }
            } else if self.new_len > self.active.len() {
                for node in &mut self.active {
                    result |= node.commit(mode, id_stack, store, messages, entry_point);
                }
                if mode != CommitMode::Unmount {
                    for _ in 0..self.new_len - self.active.len() {
                        let mut node = self.staging.pop_front().unwrap();
                        result |=
                            node.commit(CommitMode::Mount, id_stack, store, messages, entry_point);
                        self.active.push(node);
                    }
                }
            } else {
                for node in &mut self.active {
                    result |= node.commit(mode, id_stack, store, messages, entry_point);
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
        for node in &mut self.active {
            node.gc();
        }
        for node in &mut self.staging {
            node.gc();
        }
    }
}

impl<Visitor, Context, V, CS, S, M, E> Traversable<Visitor, Context, S, M, E>
    for VecStorage<ViewNode<V, CS, S, M, E>>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
    ViewNode<V, CS, S, M, E>: Traversable<Visitor, Context, S, M, E> + ViewNodeSeq<S, M, E>,
{
    fn for_each(&mut self, visitor: &mut Visitor, context: &mut Context, id_stack: &mut IdStack) {
        for node in &mut self.active {
            node.for_each(visitor, context, id_stack);
        }
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        id_stack: &mut IdStack,
    ) -> bool {
        if let Ok(index) = self.active.binary_search_by_key(&id, |node| node.id) {
            let node = &mut self.active[index];
            return node.for_id(id, visitor, context, id_stack);
        }
        false
    }
}
