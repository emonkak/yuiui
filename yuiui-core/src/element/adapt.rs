use std::fmt;

use crate::component_stack::ComponentStack;
use crate::context::{CommitContext, RenderContext};
use crate::event::{EventTarget, Lifecycle};
use crate::id::{Id, Level};
use crate::view::View;
use crate::view_node::{CommitMode, Traversable, ViewNode, ViewNodeMut, ViewNodeSeq};

use super::{Element, ElementSeq};

pub struct AdaptElement<Inner, S, M, SS, SM> {
    inner: Inner,
    select_state: fn(&S) -> &SS,
    lift_message: fn(SM) -> M,
}

impl<Inner, S, M, SS, SM> AdaptElement<Inner, S, M, SS, SM> {
    pub fn new(inner: Inner, select_state: fn(&S) -> &SS, lift_message: fn(SM) -> M) -> Self {
        Self {
            inner,
            select_state,
            lift_message,
        }
    }
}

impl<Inner, S, M, SS, SM, E> Element<S, M, E> for AdaptElement<Inner, S, M, SS, SM>
where
    Inner: Element<SS, SM, E>,
{
    type View = Adapt<Inner::View, S, M, SS, SM>;

    type Components = Adapt<Inner::Components, S, M, SS, SM>;

    fn render(
        self,
        context: &mut RenderContext<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        let mut inner_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        let inner_node = self.inner.render(&mut inner_context);
        ViewNode {
            id: inner_node.id,
            view: Adapt::new(inner_node.view, self.select_state, self.lift_message),
            pending_view: inner_node
                .pending_view
                .map(|view| Adapt::new(view, self.select_state, self.lift_message)),
            view_state: inner_node.view_state,
            children: Adapt::new(inner_node.children, self.select_state, self.lift_message),
            components: Adapt::new(inner_node.components, self.select_state, self.lift_message),
            dirty: inner_node.dirty,
        }
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, E>,
        context: &mut RenderContext<S>,
    ) -> bool {
        let mut inner_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        node.view.select_state = self.select_state;
        node.view.lift_message = self.lift_message;
        node.children.select_state = self.select_state;
        node.children.lift_message = self.lift_message;
        node.components.select_state = self.select_state;
        node.components.lift_message = self.lift_message;
        with_inner_node(node, |mut inner_node| {
            self.inner.update(&mut inner_node, &mut inner_context)
        })
    }
}

impl<Inner, S, M, SS, SM, E> ElementSeq<S, M, E> for AdaptElement<Inner, S, M, SS, SM>
where
    Inner: Element<SS, SM, E>,
{
    type Storage =
        ViewNode<Adapt<Inner::View, S, M, SS, SM>, Adapt<Inner::Components, S, M, SS, SM>, S, M, E>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        context.render_node(self)
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        context.update_node(self, storage)
    }
}

impl<Inner, S, M, SS, SM> fmt::Debug for AdaptElement<Inner, S, M, SS, SM>
where
    Inner: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("AdaptElement").field(&self.inner).finish()
    }
}

pub struct Adapt<Inner, S, M, SS, SM> {
    inner: Inner,
    select_state: fn(&S) -> &SS,
    lift_message: fn(SM) -> M,
}

impl<Inner, S, M, SS, SM> Adapt<Inner, S, M, SS, SM> {
    fn new(inner: Inner, select_state: fn(&S) -> &SS, lift_message: fn(SM) -> M) -> Self {
        Self {
            inner,
            select_state,
            lift_message,
        }
    }
}

impl<Inner, S, M, SS, SM, E> View<S, M, E> for Adapt<Inner, S, M, SS, SM>
where
    Inner: View<SS, SM, E>,
{
    type Children = Adapt<Inner::Children, S, M, SS, SM>;

    type State = Inner::State;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) {
        let inner_lifecycle = lifecycle.map(|view| view.inner);
        let mut inner_messages = Vec::new();
        let mut inner_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut inner_messages,
            entry_point: context.entry_point,
        };
        self.inner.lifecycle(
            inner_lifecycle,
            view_state,
            &mut children.inner,
            &mut inner_context,
        );
        context
            .messages
            .extend(inner_messages.into_iter().map(&self.lift_message));
    }

    fn event(
        &self,
        event: <Self as EventTarget>::Event,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) {
        let mut inner_messages = Vec::new();
        let mut inner_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut inner_messages,
            entry_point: context.entry_point,
        };
        self.inner
            .event(event, view_state, &mut children.inner, &mut inner_context);
        context
            .messages
            .extend(inner_messages.into_iter().map(&self.lift_message));
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) -> Self::State {
        let mut inner_messages = Vec::new();
        let mut inner_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut inner_messages,
            entry_point: context.entry_point,
        };
        let view_state = self.inner.build(&mut children.inner, &mut inner_context);
        context
            .messages
            .extend(inner_messages.into_iter().map(&self.lift_message));
        view_state
    }
}

impl<'event, Inner, S, M, SS, SM> EventTarget<'event> for Adapt<Inner, S, M, SS, SM>
where
    Inner: EventTarget<'event>,
{
    type Event = Inner::Event;
}

impl<Inner, S, M, SS, SM, E> ComponentStack<S, M, E> for Adapt<Inner, S, M, SS, SM>
where
    Inner: ComponentStack<SS, SM, E>,
{
    const LEVEL: Level = Inner::LEVEL;

    type View = Adapt<Inner::View, S, M, SS, SM>;

    fn force_update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        level: Level,
        context: &mut RenderContext<S>,
    ) -> bool {
        let mut inner_context = RenderContext {
            id_stack: context.id_stack,
            state: (node.components.select_state)(context.state),
        };
        with_inner_node(node, |mut inner_node| {
            Inner::force_update(&mut inner_node, level, &mut inner_context)
        })
    }
}

impl<Inner, S, M, SS, SM, E> ElementSeq<S, M, E> for Adapt<Inner, S, M, SS, SM>
where
    Inner: ElementSeq<SS, SM, E>,
{
    type Storage = Adapt<Inner::Storage, S, M, SS, SM>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        let mut inner_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        Adapt::new(
            self.inner.render_children(&mut inner_context),
            self.select_state,
            self.lift_message,
        )
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        let mut inner_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        self.inner
            .update_children(&mut storage.inner, &mut inner_context)
    }
}

impl<Inner, S, M, SS, SM, E> ViewNodeSeq<S, M, E> for Adapt<Inner, S, M, SS, SM>
where
    Inner: ViewNodeSeq<SS, SM, E>,
{
    const SIZE_HINT: (usize, Option<usize>) = Inner::SIZE_HINT;

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn commit(&mut self, mode: CommitMode, context: &mut CommitContext<S, M, E>) -> bool {
        let mut inner_messages = Vec::new();
        let mut inner_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut inner_messages,
            entry_point: context.entry_point,
        };
        let result = self.inner.commit(mode, &mut inner_context);
        context
            .messages
            .extend(inner_messages.into_iter().map(&self.lift_message));
        result
    }

    fn gc(&mut self) {
        self.inner.gc();
    }
}

impl<'context, Inner, S, M, SS, SM, Visitor> Traversable<Visitor, RenderContext<'context, S>>
    for Adapt<Inner, S, M, SS, SM>
where
    Inner: for<'inner_context> Traversable<Visitor, RenderContext<'inner_context, SS>>,
{
    fn for_each(&mut self, visitor: &mut Visitor, context: &mut RenderContext<'context, S>) {
        let mut inner_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        self.inner.for_each(visitor, &mut inner_context)
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut RenderContext<'context, S>,
    ) -> bool {
        let mut inner_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        self.inner.for_id(id, visitor, &mut inner_context)
    }
}

impl<'context, Inner, S, M, SS, SM, E, Visitor>
    Traversable<Visitor, CommitContext<'context, S, M, E>> for Adapt<Inner, S, M, SS, SM>
where
    Inner: for<'inner_context> Traversable<Visitor, CommitContext<'inner_context, SS, SM, E>>,
{
    fn for_each(&mut self, visitor: &mut Visitor, context: &mut CommitContext<'context, S, M, E>) {
        let mut inner_messages = Vec::new();
        let mut inner_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut inner_messages,
            entry_point: context.entry_point,
        };
        self.inner.for_each(visitor, &mut inner_context);
        context
            .messages
            .extend(inner_messages.into_iter().map(&self.lift_message));
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut CommitContext<'context, S, M, E>,
    ) -> bool {
        let mut inner_messages = Vec::new();
        let mut inner_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut inner_messages,
            entry_point: context.entry_point,
        };
        let result = self.inner.for_id(id, visitor, &mut inner_context);
        context
            .messages
            .extend(inner_messages.into_iter().map(&self.lift_message));
        result
    }
}

impl<Inner, S, M, SS, SM> fmt::Debug for Adapt<Inner, S, M, SS, SM>
where
    Inner: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Adapt").field(&self.inner).finish()
    }
}

fn with_inner_node<V, CS, S, M, SS, SM, E, F, T>(
    node: &mut ViewNodeMut<Adapt<V, S, M, SS, SM>, Adapt<CS, S, M, SS, SM>, S, M, E>,
    f: F,
) -> T
where
    V: View<SS, SM, E>,
    CS: ComponentStack<SS, SM, E, View = V>,
    F: FnOnce(ViewNodeMut<V, CS, SS, SM, E>) -> T,
{
    let mut inner_pending_view = node.pending_view.take().map(|view| view.inner);
    let inner_node = ViewNodeMut {
        id: node.id,
        view: &mut node.view.inner,
        pending_view: &mut inner_pending_view,
        view_state: node.view_state,
        children: &mut node.children.inner,
        components: &mut node.components.inner,
        dirty: node.dirty,
    };
    let select_state = &node.view.select_state;
    let lift_message = &node.view.lift_message;
    let result = f(inner_node);
    *node.pending_view =
        inner_pending_view.map(|view| Adapt::new(view, *select_state, *lift_message));
    result
}
