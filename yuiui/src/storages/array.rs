use std::sync::Once;

use crate::component_stack::ComponentStack;
use crate::context::{MessageContext, RenderContext};
use crate::element::Element;
use crate::element::ElementSeq;
use crate::event::{Event, EventListener, EventMask};
use crate::id::Id;
use crate::state::Store;
use crate::traversable::{Monoid, Traversable, Visitor};
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode, ViewNodeSeq};

#[derive(Debug)]
pub struct ArrayStorage<T, const N: usize> {
    nodes: [T; N],
    dirty: bool,
}

impl<T, const N: usize> ArrayStorage<T, N> {
    fn new(nodes: [T; N]) -> Self {
        Self { nodes, dirty: true }
    }
}

impl<E, S, M, B, const N: usize> ElementSeq<S, M, B> for [E; N]
where
    E: Element<S, M, B>,
{
    type Storage = ArrayStorage<ViewNode<E::View, E::Components, S, M, B>, N>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        ArrayStorage::new(self.map(|element| element.render(context, store)))
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            let node = &mut storage.nodes[i];
            has_changed |= element.update(node.borrow_mut(), context, store);
        }

        storage.dirty |= has_changed;

        has_changed
    }
}

impl<'a, V, CS, S, M, B, const N: usize> ViewNodeSeq<S, M, B>
    for ArrayStorage<ViewNode<V, CS, S, M, B>, N>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        INIT.call_once(|| unsafe {
            let mut types = Vec::new();
            <V as EventListener>::Event::collect_types(&mut types);
            if !types.is_empty() {
                EVENT_MASK.extend(types);
            }
        });

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        N
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let mut result = false;
        if self.dirty || mode.is_propagatable() {
            for node in &mut self.nodes {
                result |= node.commit(mode, context, store, backend);
            }
            self.dirty = false;
        }
        result
    }
}

impl<V, CS, S, M, B, Visitor, Context, const N: usize>
    Traversable<Visitor, Visitor::Context, Visitor::Output, S, B>
    for ArrayStorage<ViewNode<V, CS, S, M, B>, N>
where
    ViewNode<V, CS, S, M, B>: Traversable<Visitor, Context, Visitor::Output, S, B>,
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
    Visitor: self::Visitor<ViewNode<V, CS, S, M, B>, S, B, Context = Context>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        backend: &mut B,
    ) -> Visitor::Output {
        let mut result = Visitor::Output::default();
        for node in &mut self.nodes {
            result = result.combine(node.for_each(visitor, context, store, backend));
        }
        result
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        store: &Store<S>,
        backend: &mut B,
    ) -> Option<Visitor::Output> {
        if let Ok(index) = self.nodes.binary_search_by_key(&id, |node| node.id) {
            let node = &mut self.nodes[index];
            node.for_id(id, visitor, context, store, backend)
        } else {
            None
        }
    }
}
