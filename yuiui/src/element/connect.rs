use std::fmt;
use std::marker::PhantomData;

use crate::component_stack::ComponentStack;
use crate::context::{MessageContext, RenderContext};
use crate::event::{EventListener, EventMask, Lifecycle};
use crate::id::{Depth, IdPath};
use crate::state::Store;
use crate::traversable::Traversable;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode, ViewNodeMut, ViewNodeSeq};

use super::{Element, ElementSeq};

pub struct Connect<T, S, M, SS, SM> {
    target: T,
    store_selector: fn(&S) -> &Store<SS>,
    message_selector: fn(SM) -> M,
    phantom: PhantomData<(SS, SM)>,
}

impl<T, S, M, SS, SM> Connect<T, S, M, SS, SM> {
    pub fn new(
        target: T,
        store_selector: fn(&S) -> &Store<SS>,
        message_selector: fn(SM) -> M,
    ) -> Self {
        Self {
            target,
            store_selector,
            message_selector,
            phantom: PhantomData,
        }
    }
}

impl<T, S, M, SS, SM> fmt::Debug for Connect<T, S, M, SS, SM>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Connect").field(&self.target).finish()
    }
}

impl<T, S, M, SS, SM, B> Element<S, M, B> for Connect<T, S, M, SS, SM>
where
    T: Element<SS, SM, B>,
{
    type View = Connect<T::View, S, M, SS, SM>;

    type Components = Connect<T::Components, S, M, SS, SM>;

    fn render(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let sub_store = (self.store_selector)(store);
        let sub_node = self.target.render(context, sub_store);
        ViewNode {
            id: sub_node.id,
            state: sub_node.state.map(|state| {
                state.map_view(|view| {
                    Connect::new(
                        view,
                        self.store_selector.clone(),
                        self.message_selector.clone(),
                    )
                })
            }),
            children: Connect::new(
                sub_node.children,
                self.store_selector.clone(),
                self.message_selector.clone(),
            ),
            components: Connect::new(
                sub_node.components,
                self.store_selector,
                self.message_selector,
            ),
            event_mask: sub_node.event_mask,
            dirty: sub_node.dirty,
        }
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let sub_store = (self.store_selector)(store);
        with_sub_node(node, |sub_node| {
            self.target.update(sub_node, context, sub_store)
        })
    }
}

impl<T, S, M, SS, SM, B> ElementSeq<S, M, B> for Connect<T, S, M, SS, SM>
where
    T: ElementSeq<SS, SM, B>,
{
    type Storage = Connect<T::Storage, S, M, SS, SM>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        let sub_store = (self.store_selector)(store);
        Connect::new(
            self.target.render_children(context, sub_store),
            self.store_selector.clone(),
            self.message_selector.clone(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let sub_store = (self.store_selector)(store);
        self.target
            .update_children(&mut storage.target, context, sub_store)
    }
}

impl<T, S, M, SS, SM, B> ViewNodeSeq<S, M, B> for Connect<T, S, M, SS, SM>
where
    T: ViewNodeSeq<SS, SM, B>,
{
    fn event_mask() -> &'static EventMask {
        T::event_mask()
    }

    fn len(&self) -> usize {
        self.target.len()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let sub_store = (self.store_selector)(store);
        let mut sub_context = context.new_sub_context();
        let result = self
            .target
            .commit(mode, &mut sub_context, sub_store, backend);
        context.merge_sub_context(sub_context, &self.message_selector);
        result
    }
}

impl<'a, T, S, M, SS, SM, Visitor, Output, B> Traversable<Visitor, RenderContext, Output, S, B>
    for Connect<T, S, M, SS, SM>
where
    T: Traversable<Visitor, RenderContext, Output, SS, B>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &mut B,
    ) -> Output {
        let sub_store = (self.store_selector)(store);
        self.target.for_each(visitor, context, sub_store, backend)
    }

    fn for_id_path(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &mut B,
    ) -> Option<Output> {
        let sub_store = (self.store_selector)(store);
        self.target
            .for_id_path(id_path, visitor, context, sub_store, backend)
    }
}

impl<'a, T, S, M, SS, SM, Visitor, Output, B> Traversable<Visitor, MessageContext<M>, Output, S, B>
    for Connect<T, S, M, SS, SM>
where
    T: Traversable<Visitor, MessageContext<SM>, Output, SS, B>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> Output {
        let sub_store = (self.store_selector)(store);
        let mut sub_context = context.new_sub_context();
        let result = self
            .target
            .for_each(visitor, &mut sub_context, sub_store, backend);
        context.merge_sub_context(sub_context, &self.message_selector);
        result
    }

    fn for_id_path(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> Option<Output> {
        let sub_store = (self.store_selector)(store);
        let mut sub_context = context.new_sub_context();
        let result =
            self.target
                .for_id_path(id_path, visitor, &mut sub_context, sub_store, backend);
        context.merge_sub_context(sub_context, &self.message_selector);
        result
    }
}

impl<T, S, M, SS, SM, B> ComponentStack<S, M, B> for Connect<T, S, M, SS, SM>
where
    T: ComponentStack<SS, SM, B>,
{
    const LEN: usize = T::LEN;

    type View = Connect<T::View, S, M, SS, SM>;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let store_selector = &node.components.store_selector;
        let sub_store = store_selector(store);
        with_sub_node(node, |sub_node| {
            T::update(sub_node, target_depth, current_depth, context, sub_store)
        })
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let sub_store = (self.store_selector)(store);
        match mode {
            CommitMode::Mount => sub_store.subscribe(context.id_path().to_vec(), current_depth),
            CommitMode::Unmount => sub_store.unsubscribe(context.id_path(), current_depth),
            CommitMode::Update => {}
        }
        let mut sub_context = context.new_sub_context();
        let result = self.target.commit(
            mode,
            target_depth,
            current_depth,
            &mut sub_context,
            sub_store,
            backend,
        );
        context.merge_sub_context(sub_context, &self.message_selector);
        result
    }
}

impl<T, S, M, SS, SM, B> View<S, M, B> for Connect<T, S, M, SS, SM>
where
    T: View<SS, SM, B>,
{
    type Children = Connect<T::Children, S, M, SS, SM>;

    type State = T::State;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) {
        let sub_lifecycle = lifecycle.map(|view| view.target);
        let sub_store = (self.store_selector)(store);
        let mut sub_context = context.new_sub_context();
        self.target.lifecycle(
            sub_lifecycle,
            view_state,
            &children.target,
            &mut sub_context,
            sub_store,
            backend,
        );
        context.merge_sub_context(sub_context, &self.message_selector);
    }

    fn event(
        &self,
        event: <Self as EventListener>::Event,
        view_state: &mut Self::State,
        children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) {
        let sub_store = (self.store_selector)(store);
        let mut sub_context = context.new_sub_context();
        self.target.event(
            event,
            view_state,
            &children.target,
            &mut sub_context,
            sub_store,
            backend,
        );
        context.merge_sub_context(sub_context, &self.message_selector);
    }

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        store: &Store<S>,
        backend: &mut B,
    ) -> Self::State {
        let sub_store = (self.store_selector)(store);
        self.target
            .build(&children.target, sub_store, backend)
    }
}

impl<'event, T, S, M, SS, SM> EventListener<'event> for Connect<T, S, M, SS, SM>
where
    T: EventListener<'event>,
{
    type Event = T::Event;
}

fn with_sub_node<Callback, Output, S, M, SS, SM, V, CS, B>(
    node: &mut ViewNodeMut<Connect<V, S, M, SS, SM>, Connect<CS, S, M, SS, SM>, S, M, B>,
    callback: Callback,
) -> Output
where
    Callback: FnOnce(&mut ViewNodeMut<V, CS, SS, SM, B>) -> Output,
    V: View<SS, SM, B>,
    CS: ComponentStack<SS, SM, B, View = V>,
{
    let store_selector = &node.components.store_selector;
    let message_selector = &node.components.message_selector;
    let mut sub_node_state = node
        .state
        .take()
        .map(|state| state.map_view(|view| view.target));
    let mut sub_node = ViewNodeMut {
        id: node.id,
        state: &mut sub_node_state,
        children: &mut node.children.target,
        components: &mut node.components.target,
        dirty: &mut node.dirty,
    };
    let result = callback(&mut sub_node);
    *node.state = sub_node_state.map(|state| {
        state.map_view(|view| Connect::new(view, store_selector.clone(), message_selector.clone()))
    });
    result
}
