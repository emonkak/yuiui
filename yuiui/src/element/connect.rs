use std::fmt;

use crate::component_stack::ComponentStack;
use crate::event::Lifecycle;
use crate::id::{Depth, Id, IdContext};
use crate::store::Store;
use crate::view::View;
use crate::view_node::{
    CommitContext, CommitMode, RenderContext, Traversable, ViewNode, ViewNodeMut, ViewNodeSeq,
};

use super::{Element, ElementSeq};

pub struct ConnectEl<T, S, M, SS, SM> {
    target: T,
    select_store: fn(&S) -> &Store<SS>,
    lift_message: fn(SM) -> M,
}

impl<T, S, M, SS, SM> ConnectEl<T, S, M, SS, SM> {
    pub fn new(target: T, select_store: fn(&S) -> &Store<SS>, lift_message: fn(SM) -> M) -> Self {
        Self {
            target,
            select_store,
            lift_message,
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
        let sub_store = (self.select_store)(state);
        let sub_node = self.target.render(id_context, sub_store);
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
        mut node: ViewNodeMut<Self::View, Self::Components, S, M, R>,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        let sub_store = (self.select_store)(state);
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
        let sub_store = (self.select_store)(store);
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
        messages.extend(sub_messages.into_iter().map(&self.lift_message));
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
        let sub_store = (self.select_store)(store);
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
        messages.extend(sub_messages.into_iter().map(&self.lift_message));
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::State {
        let sub_store = (self.select_store)(store);
        self.target.build(&mut children.target, sub_store, renderer)
    }
}

impl<T, S, M, SS, SM, R> ComponentStack<S, M, R> for Connect<T, S, M, SS, SM>
where
    T: ComponentStack<SS, SM, R>,
{
    const LEN: usize = T::LEN;

    type View = Connect<T::View, S, M, SS, SM>;

    fn depth<'a>(node: &mut ViewNodeMut<'a, Self::View, Self, S, M, R>) -> Depth {
        with_sub_node(node, |mut sub_node| T::depth(&mut sub_node))
    }

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, R>,
        depth: Depth,
        id_context: &mut IdContext,
        store: &Store<S>,
    ) -> bool {
        let sub_store = (node.components.select_store)(store);
        with_sub_node(node, |mut sub_node| {
            T::update(&mut sub_node, depth, id_context, sub_store)
        })
    }

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, R>,
        mode: CommitMode,
        depth: Depth,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        renderer: &mut R,
    ) -> bool {
        let sub_store = (node.components.select_store)(store);
        let mut sub_messages = Vec::new();
        let result = with_sub_node(node, |mut sub_node| {
            match mode {
                CommitMode::Mount => {
                    sub_store.subscribe(id_context.id_path().to_vec(), T::depth(&mut sub_node))
                }
                CommitMode::Unmount => {
                    sub_store.unsubscribe(id_context.id_path(), T::depth(&mut sub_node))
                }
                CommitMode::Update => {}
            }
            T::commit(
                &mut sub_node,
                mode,
                depth,
                id_context,
                sub_store,
                &mut sub_messages,
                renderer,
            )
        });
        messages.extend(sub_messages.into_iter().map(&node.components.lift_message));
        result
    }
}

impl<T, S, M, SS, SM, R> ElementSeq<S, M, R> for Connect<T, S, M, SS, SM>
where
    T: ElementSeq<SS, SM, R>,
{
    type Storage = Connect<T::Storage, S, M, SS, SM>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
        let sub_store = (self.select_store)(state);
        Connect::new(
            self.target.render_children(id_context, sub_store),
            self.select_store.clone(),
            self.lift_message.clone(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        let sub_store = (self.select_store)(state);
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
        let sub_store = (self.select_store)(store);
        let mut sub_messages = Vec::new();
        let result = self
            .target
            .commit(mode, id_context, sub_store, &mut sub_messages, renderer);
        messages.extend(sub_messages.into_iter().map(&self.lift_message));
        result
    }

    fn gc(&mut self) {
        self.target.gc();
    }
}

impl<'context, T, S, M, SS, SM, R, Visitor>
    Traversable<Visitor, RenderContext<'context, S>, S, M, R> for Connect<T, S, M, SS, SM>
where
    T: for<'sub_context> Traversable<Visitor, RenderContext<'sub_context, SS>, SS, SM, R>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut RenderContext<'context, S>,
        id_context: &mut IdContext,
    ) {
        let mut sub_context = RenderContext {
            store: (self.select_store)(context.store),
        };
        self.target.for_each(visitor, &mut sub_context, id_context)
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut RenderContext<'context, S>,
        id_context: &mut IdContext,
    ) -> bool {
        let mut sub_context = RenderContext {
            store: (self.select_store)(context.store),
        };
        self.target
            .for_id(id, visitor, &mut sub_context, id_context)
    }
}

impl<'context, T, S, M, SS, SM, R, Visitor>
    Traversable<Visitor, CommitContext<'context, S, M, R>, S, M, R> for Connect<T, S, M, SS, SM>
where
    T: for<'sub_context> Traversable<Visitor, CommitContext<'sub_context, SS, SM, R>, SS, SM, R>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut CommitContext<'context, S, M, R>,
        id_context: &mut IdContext,
    ) {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            store: (self.select_store)(context.store),
            messages: &mut sub_messages,
            renderer: context.renderer,
        };
        self.target.for_each(visitor, &mut sub_context, id_context);
        context
            .messages
            .extend(sub_messages.into_iter().map(&self.lift_message));
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut CommitContext<'context, S, M, R>,
        id_context: &mut IdContext,
    ) -> bool {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            store: (self.select_store)(context.store),
            messages: &mut sub_messages,
            renderer: context.renderer,
        };
        let result = self
            .target
            .for_id(id, visitor, &mut sub_context, id_context);
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

fn with_sub_node<Callback, Output, V, CS, S, M, SS, SM, R>(
    node: &mut ViewNodeMut<Connect<V, S, M, SS, SM>, Connect<CS, S, M, SS, SM>, S, M, R>,
    callback: Callback,
) -> Output
where
    Callback: FnOnce(ViewNodeMut<V, CS, SS, SM, R>) -> Output,
    V: View<SS, SM, R>,
    CS: ComponentStack<SS, SM, R, View = V>,
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
