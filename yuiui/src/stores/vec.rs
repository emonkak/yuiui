use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Once;

use crate::component_node::ComponentStack;
use crate::context::{EffectContext, IdContext, RenderContext};
use crate::element::{Element, ElementSeq};
use crate::event::{Event, EventMask, HasEvent};
use crate::id::IdPath;
use crate::state::State;
use crate::traversable::{Traversable, TraversableVisitor};
use crate::view::View;
use crate::widget_node::{CommitMode, WidgetNode, WidgetNodeSeq};

pub struct VecStore<V: View<S, E>, CS: ComponentStack<S, E, View = V>, S: State, E> {
    active: Vec<WidgetNode<V, CS, S, E>>,
    staging: VecDeque<WidgetNode<V, CS, S, E>>,
    new_len: usize,
    dirty: bool,
}

impl<V, CS, S, E> VecStore<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn new(active: Vec<WidgetNode<V, CS, S, E>>) -> Self {
        Self {
            staging: VecDeque::with_capacity(active.len()),
            new_len: active.len(),
            active,
            dirty: true,
        }
    }
}

impl<V, CS, S, E> fmt::Debug for VecStore<V, CS, S, E>
where
    V: View<S, E> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Children as ElementSeq<S, E>>::Store: fmt::Debug,
    CS: ComponentStack<S, E, View = V> + fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VecStore")
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
    type Store = VecStore<El::View, El::Components, S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        VecStore::new(
            self.into_iter()
                .map(|element| element.render(state, env, context))
                .collect(),
        )
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let mut has_changed = false;

        store
            .staging
            .reserve_exact(self.len().saturating_sub(store.active.len()));
        store.new_len = self.len();

        for (i, element) in self.into_iter().enumerate() {
            if i < store.active.len() {
                let node = &mut store.active[i];
                has_changed |= element.update(node.scope(), state, env, context);
            } else {
                let j = i - store.active.len();
                if j < store.staging.len() {
                    let node = &mut store.staging[j];
                    has_changed |= element.update(node.scope(), state, env, context);
                } else {
                    let node = element.render(state, env, context);
                    store.staging.push_back(node);
                    has_changed = true;
                }
            }
        }

        store.dirty |= has_changed;

        has_changed
    }
}

impl<V, CS, S, E> WidgetNodeSeq<S, E> for VecStore<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        INIT.call_once(|| unsafe {
            EVENT_MASK.add_all(&<V as HasEvent>::Event::allowed_types());
        });

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        self.active.len()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            match self.new_len.cmp(&self.active.len()) {
                Ordering::Equal => {
                    // new_len == active_len
                    for node in &mut self.active {
                        node.commit(mode, state, env, context);
                    }
                }
                Ordering::Less => {
                    // new_len < active_len
                    for node in &mut self.active[..self.new_len] {
                        node.commit(mode, state, env, context);
                    }
                    for mut node in self.active.drain(self.new_len..).rev() {
                        node.commit(CommitMode::Unmount, state, env, context);
                        self.staging.push_front(node);
                    }
                }
                Ordering::Greater => {
                    // new_len > active_len
                    for node in &mut self.active {
                        node.commit(mode, state, env, context);
                    }
                    if mode != CommitMode::Unmount {
                        for _ in 0..self.active.len() - self.new_len {
                            let mut node = self.staging.pop_front().unwrap();
                            node.commit(CommitMode::Mount, state, env, context);
                            self.active.push(node);
                        }
                    }
                }
            }
            self.dirty = false;
        }
    }
}

impl<V, CS, Visitor, Context, S, E> Traversable<Visitor, Context, S, E> for VecStore<V, CS, S, E>
where
    V: View<S, E>,
    <V::Children as ElementSeq<S, E>>::Store: Traversable<Visitor, Context, S, E>,
    CS: ComponentStack<S, E, View = V>,
    Visitor: TraversableVisitor<WidgetNode<V, CS, S, E>, Context, S, E>,
    Context: IdContext,
    S: State,
{
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut Context) {
        for node in &mut self.active {
            node.for_each(visitor, state, env, context);
        }
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        if let Ok(index) = self
            .active
            .binary_search_by_key(&id_path.top_id(), |node| node.id)
        {
            let node = &mut self.active[index];
            node.search(id_path, visitor, state, env, context);
            true
        } else {
            false
        }
    }
}