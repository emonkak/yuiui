use std::any::TypeId;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::ops::ControlFlow;

use crate::component::ComponentStack;
use crate::context::{EffectContext, RenderContext};
use crate::element::Element;
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

use super::{CallbackMut, CommitMode, ElementSeq, TraversableSeq, WidgetNodeSeq};

pub struct VecStore<V: View<S, E>, CS, S: State, E> {
    active: Vec<WidgetNode<V, CS, S, E>>,
    staging: VecDeque<WidgetNode<V, CS, S, E>>,
    new_len: usize,
    dirty: bool,
}

impl<V, CS, S, E> VecStore<V, CS, S, E>
where
    V: View<S, E>,
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
    <V::Widget as Widget<S, E>>::Children: fmt::Debug,
    CS: fmt::Debug,
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

impl<EL, S, E> ElementSeq<S, E> for Vec<EL>
where
    EL: Element<S, E>,
    S: State,
{
    type Store = VecStore<EL::View, EL::Components, S, E>;

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
    CS: ComponentStack<S, E>,
    S: State,
{
    fn event_mask() -> EventMask {
        let mut event_mask = <V::Widget as Widget<S, E>>::Children::event_mask();
        event_mask.add(TypeId::of::<<V::Widget as Widget<S, E>>::Event>());
        event_mask
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

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let mut result = EventResult::Ignored;
        for node in &mut self.active {
            result = result.merge(node.event(event, state, env, context));
        }
        result
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        if let Ok(index) = self
            .active
            .binary_search_by_key(&event.id_path.top_id(), |node| node.id)
        {
            let node = &mut self.active[index];
            node.internal_event(event, state, env, context)
        } else {
            EventResult::Ignored
        }
    }
}

impl<'a, V, CS, S, E, C> TraversableSeq<C> for &'a VecStore<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
    C: CallbackMut<&'a WidgetNode<V, CS, S, E>>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        for node in &self.active {
            if let ControlFlow::Break(_) = callback.call(node) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}

impl<'a, V, CS, S, E, C> TraversableSeq<C> for &'a mut VecStore<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
    C: CallbackMut<&'a mut WidgetNode<V, CS, S, E>>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        for node in &mut self.active {
            if let ControlFlow::Break(_) = callback.call(node) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}
