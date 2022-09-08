use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Once;

use crate::component_stack::ComponentStack;
use crate::context::{EffectContext, RenderContext};
use crate::effect::EffectOps;
use crate::element::{Element, ElementSeq};
use crate::event::{Event, EventMask, HasEvent};
use crate::id::{Id, IdPath};
use crate::state::State;
use crate::traversable::{Monoid, Traversable, Visitor};
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode, ViewNodeSeq};

pub struct VecStorage<V: View<S, B>, CS: ComponentStack<S, B, View = V>, S: State, B> {
    active: Vec<ViewNode<V, CS, S, B>>,
    staging: VecDeque<ViewNode<V, CS, S, B>>,
    new_len: usize,
    dirty: bool,
}

impl<V, CS, S, B> VecStorage<V, CS, S, B>
where
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    fn new(active: Vec<ViewNode<V, CS, S, B>>) -> Self {
        Self {
            staging: VecDeque::with_capacity(active.len()),
            new_len: active.len(),
            active,
            dirty: true,
        }
    }
}

impl<V, CS, S, B> fmt::Debug for VecStorage<V, CS, S, B>
where
    V: View<S, B> + fmt::Debug,
    V::State: fmt::Debug,
    <V::Children as ElementSeq<S, B>>::Storage: fmt::Debug,
    CS: ComponentStack<S, B, View = V> + fmt::Debug,
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

impl<E, S, B> ElementSeq<S, B> for Vec<E>
where
    E: Element<S, B>,
    S: State,
{
    type Storage = VecStorage<E::View, E::Components, S, B>;

    const DEPTH: usize = E::DEPTH;

    fn render_children(self, context: &mut RenderContext, state: &S, backend: &B) -> Self::Storage {
        VecStorage::new(
            self.into_iter()
                .map(|element| element.render(context, state, backend))
                .collect(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let mut has_changed = false;

        storage
            .staging
            .reserve_exact(self.len().saturating_sub(storage.active.len()));
        storage.new_len = self.len();

        for (i, element) in self.into_iter().enumerate() {
            if i < storage.active.len() {
                let node = &mut storage.active[i];
                has_changed |= element.update(&mut node.borrow_mut(), context, state, backend);
            } else {
                let j = i - storage.active.len();
                if j < storage.staging.len() {
                    let node = &mut storage.staging[j];
                    has_changed |= element.update(&mut node.borrow_mut(), context, state, backend);
                } else {
                    let node = element.render(context, state, backend);
                    storage.staging.push_back(node);
                    has_changed = true;
                }
            }
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<V, CS, S, B> ViewNodeSeq<S, B> for VecStorage<V, CS, S, B>
where
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
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
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S> {
        let mut result = EffectOps::nop();
        if self.dirty || mode.is_propagatable() {
            match self.new_len.cmp(&self.active.len()) {
                Ordering::Equal => {
                    // new_len == active_len
                    for node in &mut self.active {
                        result = result.combine(node.commit(mode, context, state, backend));
                    }
                }
                Ordering::Less => {
                    // new_len < active_len
                    for node in &mut self.active[..self.new_len] {
                        result = result.combine(node.commit(mode, context, state, backend));
                    }
                    for mut node in self.active.drain(self.new_len..).rev() {
                        result = result.combine(node.commit(
                            CommitMode::Unmount,
                            context,
                            state,
                            backend,
                        ));
                        self.staging.push_front(node);
                    }
                }
                Ordering::Greater => {
                    // new_len > active_len
                    for node in &mut self.active {
                        result = result.combine(node.commit(mode, context, state, backend));
                    }
                    if mode != CommitMode::Unmount {
                        for _ in 0..self.active.len() - self.new_len {
                            let mut node = self.staging.pop_front().unwrap();
                            result = result.combine(node.commit(
                                CommitMode::Mount,
                                context,
                                state,
                                backend,
                            ));
                            self.active.push(node);
                        }
                    }
                }
            }
            self.dirty = false;
        }
        result
    }
}

impl<V, CS, Visitor, Context, S, B> Traversable<Visitor, Context, Visitor::Output, S, B>
    for VecStorage<V, CS, S, B>
where
    ViewNode<V, CS, S, B>: Traversable<Visitor, Context, Visitor::Output, S, B>,
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
    Visitor: self::Visitor<ViewNode<V, CS, S, B>, Context, S, B>,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Visitor::Output {
        let mut result = Visitor::Output::default();
        for node in &mut self.active {
            result = result.combine(node.for_each(visitor, context, state, backend));
        }
        result
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut Context,
        state: &S,
        backend: &B,
    ) -> Option<Visitor::Output> {
        let id = Id::from_bottom(id_path);
        if let Ok(index) = self.active.binary_search_by_key(&id, |node| node.id) {
            let node = &mut self.active[index];
            node.search(id_path, visitor, context, state, backend)
        } else {
            None
        }
    }
}
