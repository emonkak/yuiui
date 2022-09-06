use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Once;

use crate::component_stack::ComponentStack;
use crate::context::{EffectContext, IdContext, RenderContext};
use crate::element::{Element, ElementSeq};
use crate::event::{Event, EventMask, HasEvent};
use crate::id::{Id, IdPath};
use crate::state::State;
use crate::traversable::{Traversable, TraversableVisitor};
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode, ViewNodeSeq};

pub struct VecStorage<V: View<S, E>, CS: ComponentStack<S, E, View = V>, S: State, E> {
    active: Vec<ViewNode<V, CS, S, E>>,
    staging: VecDeque<ViewNode<V, CS, S, E>>,
    new_len: usize,
    dirty: bool,
}

impl<V, CS, S, E> VecStorage<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn new(active: Vec<ViewNode<V, CS, S, E>>) -> Self {
        Self {
            staging: VecDeque::with_capacity(active.len()),
            new_len: active.len(),
            active,
            dirty: true,
        }
    }
}

impl<V, CS, S, E> fmt::Debug for VecStorage<V, CS, S, E>
where
    V: View<S, E> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Children as ElementSeq<S, E>>::Storage: fmt::Debug,
    CS: ComponentStack<S, E, View = V> + fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VecStorage")
            .field("active", &self.active)
            .field("staging", &self.staging)
            .field("new_len", &self.new_len)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<El, S, E> ElementSeq<S, E> for Vec<El>
where
    El: Element<S, E>,
    S: State,
{
    type Storage = VecStorage<El::View, El::Components, S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage {
        VecStorage::new(
            self.into_iter()
                .map(|element| element.render(state, env, context))
                .collect(),
        )
    }

    fn update(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let mut has_changed = false;

        storage
            .staging
            .reserve_exact(self.len().saturating_sub(storage.active.len()));
        storage.new_len = self.len();

        for (i, element) in self.into_iter().enumerate() {
            if i < storage.active.len() {
                let node = &mut storage.active[i];
                has_changed |= element.update(node.scope(), state, env, context);
            } else {
                let j = i - storage.active.len();
                if j < storage.staging.len() {
                    let node = &mut storage.staging[j];
                    has_changed |= element.update(node.scope(), state, env, context);
                } else {
                    let node = element.render(state, env, context);
                    storage.staging.push_back(node);
                    has_changed = true;
                }
            }
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<V, CS, S, E> ViewNodeSeq<S, E> for VecStorage<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        INIT.call_once(|| unsafe {
            let mut types = Vec::new();
            <V as HasEvent>::Event::collect_types(&mut types);
            EVENT_MASK.add_all(&types);
        });

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        self.active.len()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        if self.dirty || mode.is_propagatable() {
            let mut has_changed = false;
            match self.new_len.cmp(&self.active.len()) {
                Ordering::Equal => {
                    // new_len == active_len
                    for node in &mut self.active {
                        has_changed |= node.commit(mode, state, env, context);
                    }
                }
                Ordering::Less => {
                    // new_len < active_len
                    for node in &mut self.active[..self.new_len] {
                        has_changed |= node.commit(mode, state, env, context);
                    }
                    for mut node in self.active.drain(self.new_len..).rev() {
                        has_changed |= node.commit(CommitMode::Unmount, state, env, context);
                        self.staging.push_front(node);
                    }
                }
                Ordering::Greater => {
                    // new_len > active_len
                    for node in &mut self.active {
                        has_changed |= node.commit(mode, state, env, context);
                    }
                    if mode != CommitMode::Unmount {
                        for _ in 0..self.active.len() - self.new_len {
                            let mut node = self.staging.pop_front().unwrap();
                            has_changed |= node.commit(CommitMode::Mount, state, env, context);
                            self.active.push(node);
                        }
                    }
                }
            }
            self.dirty = false;
            has_changed
        } else {
            false
        }
    }
}

impl<V, CS, Visitor, Context, S, E> Traversable<Visitor, Context, S, E> for VecStorage<V, CS, S, E>
where
    V: View<S, E>,
    <V::Children as ElementSeq<S, E>>::Storage: Traversable<Visitor, Context, S, E>,
    CS: ComponentStack<S, E, View = V>,
    Visitor: TraversableVisitor<ViewNode<V, CS, S, E>, Context, S, E>,
    Context: IdContext,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        let mut result = false;
        for node in &mut self.active {
            result |= node.for_each(visitor, state, env, context);
        }
        result
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        let id = id_path.last().copied().unwrap_or(Id::ROOT);
        if let Ok(index) = self.active.binary_search_by_key(&id, |node| node.id) {
            let node = &mut self.active[index];
            node.search(id_path, visitor, state, env, context);
            true
        } else {
            false
        }
    }
}
