use std::fmt;

use crate::component_stack::ComponentStack;
use crate::event::{EventTarget, Lifecycle};
use crate::id::{Depth, Id, IdStack};
use crate::store::Store;
use crate::view::View;
use crate::view_node::{
    CommitContext, CommitMode, RenderContext, Traversable, ViewNode, ViewNodeMut, ViewNodeSeq,
};

use super::{Element, ElementSeq};

pub struct ConnectElement<T, S, M, SS, SM> {
    target: T,
    select_store: fn(&S) -> &Store<SS>,
    lift_message: fn(SM) -> M,
}

impl<T, S, M, SS, SM> ConnectElement<T, S, M, SS, SM> {
    pub fn new(target: T, select_store: fn(&S) -> &Store<SS>, lift_message: fn(SM) -> M) -> Self {
        Self {
            target,
            select_store,
            lift_message,
        }
    }
}

impl<T, S, M, SS, SM, E> Element<S, M, E> for ConnectElement<T, S, M, SS, SM>
where
    T: Element<SS, SM, E>,
{
    type View = Connect<T::View, S, M, SS, SM>;

    type Components = Connect<T::Components, S, M, SS, SM>;

    fn render(
        self,
        id_stack: &mut IdStack,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        let sub_store = (self.select_store)(state);
        let sub_node = self.target.render(id_stack, sub_store.state());
        ViewNode {
            id: sub_node.id,
            depth: sub_node.depth,
            view: Connect::new(
                sub_node.view,
                self.select_store.clone(),
                self.lift_message.clone(),
            ),
            pending_view: sub_node.pending_view.map(|view| {
                Connect::new(view, self.select_store.clone(), self.lift_message.clone())
            }),
            state: sub_node.state,
            children: Connect::new(
                sub_node.children,
                self.select_store.clone(),
                self.lift_message.clone(),
            ),
            components: Connect::new(sub_node.components, self.select_store, self.lift_message),
            dirty: sub_node.dirty,
        }
    }

    fn update(
        self,
        mut node: ViewNodeMut<Self::View, Self::Components, S, M, E>,
        id_stack: &mut IdStack,
        state: &S,
    ) -> bool {
        let sub_store = (self.select_store)(state);
        with_sub_node(&mut node, |sub_node| {
            self.target.update(sub_node, id_stack, sub_store.state())
        })
    }
}

impl<T, S, M, SS, SM, E> ElementSeq<S, M, E> for ConnectElement<T, S, M, SS, SM>
where
    T: Element<SS, SM, E>,
{
    type Storage =
        ViewNode<Connect<T::View, S, M, SS, SM>, Connect<T::Components, S, M, SS, SM>, S, M, E>;

    fn render_children(self, id_stack: &mut IdStack, state: &S) -> Self::Storage {
        self.render(id_stack, state)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_stack: &mut IdStack,
        state: &S,
    ) -> bool {
        self.update(storage.into(), id_stack, state)
    }
}

impl<T, S, M, SS, SM> fmt::Debug for ConnectElement<T, S, M, SS, SM>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("ConnectElement").field(&self.target).finish()
    }
}

pub struct Connect<T, S, M, SS, SM> {
    target: T,
    select_store: fn(&S) -> &Store<SS>,
    lift_message: fn(SM) -> M,
}

impl<T, S, M, SS, SM> Connect<T, S, M, SS, SM> {
    fn new(target: T, select_store: fn(&S) -> &Store<SS>, lift_message: fn(SM) -> M) -> Self {
        Self {
            target,
            select_store,
            lift_message,
        }
    }
}

impl<T, S, M, SS, SM, E> View<S, M, E> for Connect<T, S, M, SS, SM>
where
    T: View<SS, SM, E>,
{
    type Children = Connect<T::Children, S, M, SS, SM>;

    type State = T::State;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        id_stack: &mut IdStack,
        store: &Store<S>,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) {
        let sub_lifecycle = lifecycle.map(|view| view.target);
        let sub_store = (self.select_store)(store.state());
        let mut sub_messages = Vec::new();
        self.target.lifecycle(
            sub_lifecycle,
            state,
            &mut children.target,
            id_stack,
            sub_store,
            &mut sub_messages,
            entry_point,
        );
        messages.extend(sub_messages.into_iter().map(&self.lift_message));
    }

    fn event(
        &self,
        event: <Self as EventTarget>::Event,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        id_stack: &mut IdStack,
        store: &Store<S>,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) {
        let sub_store = (self.select_store)(store.state());
        let mut sub_messages = Vec::new();
        self.target.event(
            event,
            state,
            &mut children.target,
            id_stack,
            sub_store,
            &mut sub_messages,
            entry_point,
        );
        messages.extend(sub_messages.into_iter().map(&self.lift_message));
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        store: &Store<S>,
        entry_point: &E,
    ) -> Self::State {
        let sub_store = (self.select_store)(store.state());
        self.target
            .build(&mut children.target, sub_store, entry_point)
    }
}

impl<'event, T, S, M, SS, SM> EventTarget<'event> for Connect<T, S, M, SS, SM>
where
    T: EventTarget<'event>,
{
    type Event = T::Event;
}

impl<T, S, M, SS, SM, E> ComponentStack<S, M, E> for Connect<T, S, M, SS, SM>
where
    T: ComponentStack<SS, SM, E>,
{
    const LEN: usize = T::LEN;

    type View = Connect<T::View, S, M, SS, SM>;

    fn depth<'a>(node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>) -> Depth {
        with_sub_node(node, |mut sub_node| T::depth(&mut sub_node))
    }

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        depth: Depth,
        id_stack: &mut IdStack,
        store: &Store<S>,
    ) -> bool {
        let sub_store = (node.components.select_store)(store.state());
        with_sub_node(node, |mut sub_node| {
            T::update(&mut sub_node, depth, id_stack, sub_store)
        })
    }

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        depth: Depth,
        id_stack: &mut IdStack,
        store: &Store<S>,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool {
        let sub_store = (node.components.select_store)(store.state());
        let mut sub_messages = Vec::new();
        let result = with_sub_node(node, |mut sub_node| {
            match mode {
                CommitMode::Mount => {
                    sub_store.subscribe(id_stack.id_path().to_vec(), T::depth(&mut sub_node))
                }
                CommitMode::Unmount => {
                    sub_store.unsubscribe(id_stack.id_path(), T::depth(&mut sub_node))
                }
                CommitMode::Update => {}
            }
            T::commit(
                &mut sub_node,
                mode,
                depth,
                id_stack,
                sub_store,
                &mut sub_messages,
                entry_point,
            )
        });
        messages.extend(sub_messages.into_iter().map(&node.components.lift_message));
        result
    }
}

impl<T, S, M, SS, SM, E> ElementSeq<S, M, E> for Connect<T, S, M, SS, SM>
where
    T: ElementSeq<SS, SM, E>,
{
    type Storage = Connect<T::Storage, S, M, SS, SM>;

    fn render_children(self, id_stack: &mut IdStack, state: &S) -> Self::Storage {
        let sub_store = (self.select_store)(state);
        Connect::new(
            self.target.render_children(id_stack, sub_store.state()),
            self.select_store.clone(),
            self.lift_message.clone(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_stack: &mut IdStack,
        state: &S,
    ) -> bool {
        let sub_store = (self.select_store)(state);
        self.target
            .update_children(&mut storage.target, id_stack, sub_store.state())
    }
}

impl<T, S, M, SS, SM, E> ViewNodeSeq<S, M, E> for Connect<T, S, M, SS, SM>
where
    T: ViewNodeSeq<SS, SM, E>,
{
    const SIZE_HINT: (usize, Option<usize>) = T::SIZE_HINT;

    fn len(&self) -> usize {
        self.target.len()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        id_stack: &mut IdStack,
        store: &Store<S>,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool {
        let sub_store = (self.select_store)(store.state());
        let mut sub_messages = Vec::new();
        let result = self
            .target
            .commit(mode, id_stack, sub_store, &mut sub_messages, entry_point);
        messages.extend(sub_messages.into_iter().map(&self.lift_message));
        result
    }

    fn gc(&mut self) {
        self.target.gc();
    }
}

impl<'context, T, S, M, SS, SM, E, Visitor>
    Traversable<Visitor, RenderContext<'context, S>, S, M, E> for Connect<T, S, M, SS, SM>
where
    T: for<'sub_context> Traversable<Visitor, RenderContext<'sub_context, SS>, SS, SM, E>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut RenderContext<'context, S>,
        id_stack: &mut IdStack,
    ) {
        let mut sub_context = RenderContext {
            store: (self.select_store)(context.store.state()),
        };
        self.target.for_each(visitor, &mut sub_context, id_stack)
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut RenderContext<'context, S>,
        id_stack: &mut IdStack,
    ) -> bool {
        let mut sub_context = RenderContext {
            store: (self.select_store)(context.store.state()),
        };
        self.target.for_id(id, visitor, &mut sub_context, id_stack)
    }
}

impl<'context, T, S, M, SS, SM, E, Visitor>
    Traversable<Visitor, CommitContext<'context, S, M, E>, S, M, E> for Connect<T, S, M, SS, SM>
where
    T: for<'sub_context> Traversable<Visitor, CommitContext<'sub_context, SS, SM, E>, SS, SM, E>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut CommitContext<'context, S, M, E>,
        id_stack: &mut IdStack,
    ) {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            store: (self.select_store)(context.store.state()),
            messages: &mut sub_messages,
            entry_point: context.entry_point,
        };
        self.target.for_each(visitor, &mut sub_context, id_stack);
        context
            .messages
            .extend(sub_messages.into_iter().map(&self.lift_message));
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut CommitContext<'context, S, M, E>,
        id_stack: &mut IdStack,
    ) -> bool {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            store: (self.select_store)(context.store.state()),
            messages: &mut sub_messages,
            entry_point: context.entry_point,
        };
        let result = self.target.for_id(id, visitor, &mut sub_context, id_stack);
        context
            .messages
            .extend(sub_messages.into_iter().map(&self.lift_message));
        result
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

fn with_sub_node<Callback, Output, V, CS, S, M, SS, SM, E>(
    node: &mut ViewNodeMut<Connect<V, S, M, SS, SM>, Connect<CS, S, M, SS, SM>, S, M, E>,
    callback: Callback,
) -> Output
where
    Callback: FnOnce(ViewNodeMut<V, CS, SS, SM, E>) -> Output,
    V: View<SS, SM, E>,
    CS: ComponentStack<SS, SM, E, View = V>,
{
    let select_store = &node.components.select_store;
    let lift_message = &node.components.lift_message;
    let mut sub_pending_view = node.pending_view.take().map(|view| view.target);
    let sub_node = ViewNodeMut {
        id: node.id,
        depth: node.depth,
        view: &mut node.view.target,
        pending_view: &mut sub_pending_view,
        state: node.state,
        children: &mut node.children.target,
        components: &mut node.components.target,
        dirty: &mut node.dirty,
    };
    let result = callback(sub_node);
    *node.pending_view =
        sub_pending_view.map(|view| Connect::new(view, select_store.clone(), lift_message.clone()));
    result
}
