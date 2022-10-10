use std::fmt;
use std::marker::PhantomData;

use crate::component_stack::ComponentStack;
use crate::event::Lifecycle;
use crate::id::{Depth, Id, IdContext};
use crate::store::Store;
use crate::traversable::Traversable;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode, ViewNodeMut, ViewNodeSeq};

use super::{Element, ElementSeq};

pub struct ConnectEl<T, S, M, SS, SM> {
    target: T,
    store_selector: fn(&S) -> &Store<SS>,
    message_selector: fn(SM) -> M,
    _phantom: PhantomData<(SS, SM)>,
}

impl<T, S, M, SS, SM> ConnectEl<T, S, M, SS, SM> {
    pub fn new(
        target: T,
        store_selector: fn(&S) -> &Store<SS>,
        message_selector: fn(SM) -> M,
    ) -> Self {
        Self {
            target,
            store_selector,
            message_selector,
            _phantom: PhantomData,
        }
    }
}

impl<T, S, M, SS, SM, R> Element<S, M, R> for ConnectEl<T, S, M, SS, SM>
where
    T: Element<SS, SM, R>,
{
    type View = Connect<T::View, S, M, SS, SM>;

    type Components = Connect<T::Components, S, M, SS, SM>;

    fn render(
        self,
        id_context: &mut IdContext,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, R> {
        let sub_store = (self.store_selector)(state);
        let sub_node = self.target.render(id_context, sub_store);
        ViewNode {
            id: sub_node.id,
            view: Connect::new(
                sub_node.view,
                self.store_selector.clone(),
                self.message_selector.clone(),
            ),
            pending_view: sub_node.pending_view.map(|view| {
                Connect::new(
                    view,
                    self.store_selector.clone(),
                    self.message_selector.clone(),
                )
            }),
            state: sub_node.state,
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
            dirty: sub_node.dirty,
        }
    }

    fn update(
        self,
        mut node: ViewNodeMut<Self::View, Self::Components, S, M, R>,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        let sub_store = (self.store_selector)(state);
        with_sub_node(&mut node, |sub_node| {
            self.target.update(sub_node, id_context, sub_store)
        })
    }
}

impl<T, S, M, SS, SM, R> ElementSeq<S, M, R> for ConnectEl<T, S, M, SS, SM>
where
    T: Element<SS, SM, R>,
{
    type Storage =
        ViewNode<Connect<T::View, S, M, SS, SM>, Connect<T::Components, S, M, SS, SM>, S, M, R>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
        self.render(id_context, state)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        self.update(storage.into(), id_context, state)
    }
}

impl<T, S, M, SS, SM> fmt::Debug for ConnectEl<T, S, M, SS, SM>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("ConnectEl").field(&self.target).finish()
    }
}

pub struct Connect<T, S, M, SS, SM> {
    target: T,
    store_selector: fn(&S) -> &Store<SS>,
    message_selector: fn(SM) -> M,
    _phantom: PhantomData<(SS, SM)>,
}

impl<T, S, M, SS, SM> Connect<T, S, M, SS, SM> {
    fn new(target: T, store_selector: fn(&S) -> &Store<SS>, message_selector: fn(SM) -> M) -> Self {
        Self {
            target,
            store_selector,
            message_selector,
            _phantom: PhantomData,
        }
    }
}

impl<T, S, M, SS, SM, R> View<S, M, R> for Connect<T, S, M, SS, SM>
where
    T: View<SS, SM, R>,
{
    type Children = Connect<T::Children, S, M, SS, SM>;

    type State = T::State;

    type Event = T::Event;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        renderer: &mut R,
    ) {
        let sub_lifecycle = lifecycle.map(|view| view.target);
        let sub_store = (self.store_selector)(store);
        let mut sub_messages = Vec::new();
        self.target.lifecycle(
            sub_lifecycle,
            state,
            &mut children.target,
            id_context,
            sub_store,
            &mut sub_messages,
            renderer,
        );
        messages.extend(sub_messages.into_iter().map(&self.message_selector));
    }

    fn event(
        &self,
        event: &Self::Event,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        renderer: &mut R,
    ) {
        let sub_store = (self.store_selector)(store);
        let mut sub_messages = Vec::new();
        self.target.event(
            event,
            state,
            &mut children.target,
            id_context,
            sub_store,
            &mut sub_messages,
            renderer,
        );
        messages.extend(sub_messages.into_iter().map(&self.message_selector));
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::State {
        let sub_store = (self.store_selector)(store);
        self.target.build(&mut children.target, sub_store, renderer)
    }
}

impl<T, S, M, SS, SM, R> ComponentStack<S, M, R> for Connect<T, S, M, SS, SM>
where
    T: ComponentStack<SS, SM, R>,
{
    const LEN: usize = T::LEN;

    type View = Connect<T::View, S, M, SS, SM>;

    fn update<'a>(
        mut node: ViewNodeMut<'a, Self::View, Self, S, M, R>,
        target_depth: Depth,
        current_depth: Depth,
        id_context: &mut IdContext,
        store: &Store<S>,
    ) -> bool {
        let sub_store = (node.components.store_selector)(store);
        with_sub_node(&mut node, |sub_node| {
            T::update(sub_node, target_depth, current_depth, id_context, sub_store)
        })
    }

    fn commit<'a>(
        mut node: ViewNodeMut<'a, Self::View, Self, S, M, R>,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        renderer: &mut R,
    ) -> bool {
        let sub_store = (node.components.store_selector)(store);
        match mode {
            CommitMode::Mount => sub_store.subscribe(id_context.id_path().to_vec(), current_depth),
            CommitMode::Unmount => sub_store.unsubscribe(id_context.id_path(), current_depth),
            CommitMode::Update => {}
        }
        let mut sub_messages = Vec::new();
        let result = with_sub_node(&mut node, |sub_node| {
            T::commit(
                sub_node,
                mode,
                target_depth,
                current_depth,
                id_context,
                sub_store,
                &mut sub_messages,
                renderer,
            )
        });
        messages.extend(
            sub_messages
                .into_iter()
                .map(&node.components.message_selector),
        );
        result
    }
}

impl<T, S, M, SS, SM, R> ElementSeq<S, M, R> for Connect<T, S, M, SS, SM>
where
    T: ElementSeq<SS, SM, R>,
{
    type Storage = Connect<T::Storage, S, M, SS, SM>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
        let sub_store = (self.store_selector)(state);
        Connect::new(
            self.target.render_children(id_context, sub_store),
            self.store_selector.clone(),
            self.message_selector.clone(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        let sub_store = (self.store_selector)(state);
        self.target
            .update_children(&mut storage.target, id_context, sub_store)
    }
}

impl<T, S, M, SS, SM, R> ViewNodeSeq<S, M, R> for Connect<T, S, M, SS, SM>
where
    T: ViewNodeSeq<SS, SM, R>,
{
    const SIZE_HINT: (usize, Option<usize>) = T::SIZE_HINT;

    fn len(&self) -> usize {
        self.target.len()
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        self.target.id_range()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        renderer: &mut R,
    ) -> bool {
        let sub_store = (self.store_selector)(store);
        let mut sub_messages = Vec::new();
        let result = self
            .target
            .commit(mode, id_context, sub_store, &mut sub_messages, renderer);
        messages.extend(sub_messages.into_iter().map(&self.message_selector));
        result
    }

    fn gc(&mut self) {
        self.target.gc();
    }
}

impl<'a, T, S, M, SS, SM, Visitor, R> Traversable<Visitor, (), S, M, R> for Connect<T, S, M, SS, SM>
where
    T: Traversable<Visitor, (), SS, SM, R>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) {
        let sub_store = (self.store_selector)(store);
        self.target
            .for_each(visitor, id_context, sub_store, renderer)
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Option<()> {
        let sub_store = (self.store_selector)(store);
        self.target
            .for_id(id, visitor, id_context, sub_store, renderer)
    }
}

impl<'a, T, S, M, SS, SM, Visitor, R> Traversable<Visitor, Vec<M>, S, M, R>
    for Connect<T, S, M, SS, SM>
where
    T: Traversable<Visitor, Vec<SM>, SS, SM, R>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Vec<M> {
        let sub_store = (self.store_selector)(store);
        self.target
            .for_each(visitor, id_context, sub_store, renderer)
            .into_iter()
            .map(&self.message_selector)
            .collect()
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        id_context: &mut IdContext,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Option<Vec<M>> {
        let sub_store = (self.store_selector)(store);
        self.target
            .for_id(id, visitor, id_context, sub_store, renderer)
            .map(|messages| messages.into_iter().map(&self.message_selector).collect())
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

fn with_sub_node<Callback, Output, V, CS, S, M, SS, SM, R>(
    node: &mut ViewNodeMut<Connect<V, S, M, SS, SM>, Connect<CS, S, M, SS, SM>, S, M, R>,
    callback: Callback,
) -> Output
where
    Callback: FnOnce(ViewNodeMut<V, CS, SS, SM, R>) -> Output,
    V: View<SS, SM, R>,
    CS: ComponentStack<SS, SM, R, View = V>,
{
    let store_selector = &node.components.store_selector;
    let message_selector = &node.components.message_selector;
    let mut sub_pending_view = node.pending_view.take().map(|view| view.target);
    let sub_node = ViewNodeMut {
        id: node.id,
        view: &mut node.view.target,
        pending_view: &mut sub_pending_view,
        state: node.state,
        children: &mut node.children.target,
        components: &mut node.components.target,
        dirty: &mut node.dirty,
    };
    let result = callback(sub_node);
    *node.pending_view = sub_pending_view
        .map(|view| Connect::new(view, store_selector.clone(), message_selector.clone()));
    result
}
