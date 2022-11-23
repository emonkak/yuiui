use std::fmt;

use crate::component_stack::ComponentStack;
use crate::context::{CommitContext, RenderContext};
use crate::event::{EventTarget, Lifecycle};
use crate::id::{Id, Level};
use crate::view::View;
use crate::view_node::{CommitMode, Traversable, ViewNode, ViewNodeMut, ViewNodeSeq};

use super::{Element, ElementSeq};

pub struct AdaptElement<T, S, M, SS, SM> {
    target: T,
    select_state: fn(&S) -> &SS,
    lift_message: fn(SM) -> M,
}

impl<T, S, M, SS, SM> AdaptElement<T, S, M, SS, SM> {
    pub fn new(target: T, select_state: fn(&S) -> &SS, lift_message: fn(SM) -> M) -> Self {
        Self {
            target,
            select_state,
            lift_message,
        }
    }
}

impl<T, S, M, SS, SM, E> Element<S, M, E> for AdaptElement<T, S, M, SS, SM>
where
    T: Element<SS, SM, E>,
{
    type View = Adapt<T::View, S, M, SS, SM>;

    type Components = Adapt<T::Components, S, M, SS, SM>;

    fn render(
        self,
        context: &mut RenderContext<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        let mut sub_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        let sub_node = self.target.render(&mut sub_context);
        ViewNode {
            id: sub_node.id,
            view: Adapt::new(
                sub_node.view,
                self.select_state.clone(),
                self.lift_message.clone(),
            ),
            pending_view: sub_node
                .pending_view
                .map(|view| Adapt::new(view, self.select_state.clone(), self.lift_message.clone())),
            view_state: sub_node.view_state,
            children: Adapt::new(
                sub_node.children,
                self.select_state.clone(),
                self.lift_message.clone(),
            ),
            components: Adapt::new(sub_node.components, self.select_state, self.lift_message),
            dirty: sub_node.dirty,
        }
    }

    fn update(
        self,
        mut node: ViewNodeMut<Self::View, Self::Components, S, M, E>,
        context: &mut RenderContext<S>,
    ) -> bool {
        let mut sub_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        with_sub_node(&mut node, |sub_node| {
            self.target.update(sub_node, &mut sub_context)
        })
    }
}

impl<T, S, M, SS, SM, E> ElementSeq<S, M, E> for AdaptElement<T, S, M, SS, SM>
where
    T: Element<SS, SM, E>,
{
    type Storage =
        ViewNode<Adapt<T::View, S, M, SS, SM>, Adapt<T::Components, S, M, SS, SM>, S, M, E>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        context.render_element(self)
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        context.update_node(self, storage)
    }
}

impl<T, S, M, SS, SM> fmt::Debug for AdaptElement<T, S, M, SS, SM>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("AdaptElement").field(&self.target).finish()
    }
}

pub struct Adapt<T, S, M, SS, SM> {
    target: T,
    select_state: fn(&S) -> &SS,
    lift_message: fn(SM) -> M,
}

impl<T, S, M, SS, SM> Adapt<T, S, M, SS, SM> {
    fn new(target: T, select_state: fn(&S) -> &SS, lift_message: fn(SM) -> M) -> Self {
        Self {
            target,
            select_state,
            lift_message,
        }
    }
}

impl<T, S, M, SS, SM, E> View<S, M, E> for Adapt<T, S, M, SS, SM>
where
    T: View<SS, SM, E>,
{
    type Children = Adapt<T::Children, S, M, SS, SM>;

    type State = T::State;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) {
        let sub_lifecycle = lifecycle.map(|view| view.target);
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut sub_messages,
            entry_point: context.entry_point,
        };
        self.target.lifecycle(
            sub_lifecycle,
            view_state,
            &mut children.target,
            &mut sub_context,
        );
        context
            .messages
            .extend(sub_messages.into_iter().map(&self.lift_message));
    }

    fn event(
        &self,
        event: <Self as EventTarget>::Event,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut sub_messages,
            entry_point: context.entry_point,
        };
        self.target
            .event(event, view_state, &mut children.target, &mut sub_context);
        context
            .messages
            .extend(sub_messages.into_iter().map(&self.lift_message));
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) -> Self::State {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut sub_messages,
            entry_point: context.entry_point,
        };
        let view_state = self.target.build(&mut children.target, &mut sub_context);
        context
            .messages
            .extend(sub_messages.into_iter().map(&self.lift_message));
        view_state
    }
}

impl<'event, T, S, M, SS, SM> EventTarget<'event> for Adapt<T, S, M, SS, SM>
where
    T: EventTarget<'event>,
{
    type Event = T::Event;
}

impl<T, S, M, SS, SM, E> ComponentStack<S, M, E> for Adapt<T, S, M, SS, SM>
where
    T: ComponentStack<SS, SM, E>,
{
    const LEVEL: Level = T::LEVEL;

    type View = Adapt<T::View, S, M, SS, SM>;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        level: Level,
        context: &mut RenderContext<S>,
    ) -> bool {
        let mut sub_context = RenderContext {
            id_stack: context.id_stack,
            state: (node.components.select_state)(context.state),
        };
        with_sub_node(node, |mut sub_node| {
            T::update(&mut sub_node, level, &mut sub_context)
        })
    }

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        level: Level,
        context: &mut CommitContext<S, M, E>,
    ) -> bool {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            id_stack: context.id_stack,
            state: (node.components.select_state)(context.state),
            messages: &mut sub_messages,
            entry_point: context.entry_point,
        };
        let result = with_sub_node(node, |mut sub_node| {
            T::commit(&mut sub_node, mode, level, &mut sub_context)
        });
        context
            .messages
            .extend(sub_messages.into_iter().map(&node.components.lift_message));
        result
    }
}

impl<T, S, M, SS, SM, E> ElementSeq<S, M, E> for Adapt<T, S, M, SS, SM>
where
    T: ElementSeq<SS, SM, E>,
{
    type Storage = Adapt<T::Storage, S, M, SS, SM>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        let mut sub_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        Adapt::new(
            self.target.render_children(&mut sub_context),
            self.select_state.clone(),
            self.lift_message.clone(),
        )
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        let mut sub_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        self.target
            .update_children(&mut storage.target, &mut sub_context)
    }
}

impl<T, S, M, SS, SM, E> ViewNodeSeq<S, M, E> for Adapt<T, S, M, SS, SM>
where
    T: ViewNodeSeq<SS, SM, E>,
{
    const SIZE_HINT: (usize, Option<usize>) = T::SIZE_HINT;

    fn len(&self) -> usize {
        self.target.len()
    }

    fn commit(&mut self, mode: CommitMode, context: &mut CommitContext<S, M, E>) -> bool {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut sub_messages,
            entry_point: context.entry_point,
        };
        let result = self.target.commit(mode, &mut sub_context);
        context
            .messages
            .extend(sub_messages.into_iter().map(&self.lift_message));
        result
    }

    fn gc(&mut self) {
        self.target.gc();
    }
}

impl<'context, T, S, M, SS, SM, Visitor> Traversable<Visitor, RenderContext<'context, S>>
    for Adapt<T, S, M, SS, SM>
where
    T: for<'sub_context> Traversable<Visitor, RenderContext<'sub_context, SS>>,
{
    fn for_each(&mut self, visitor: &mut Visitor, context: &mut RenderContext<'context, S>) {
        let mut sub_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        self.target.for_each(visitor, &mut sub_context)
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut RenderContext<'context, S>,
    ) -> bool {
        let mut sub_context = RenderContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
        };
        self.target.for_id(id, visitor, &mut sub_context)
    }
}

impl<'context, T, S, M, SS, SM, E, Visitor> Traversable<Visitor, CommitContext<'context, S, M, E>>
    for Adapt<T, S, M, SS, SM>
where
    T: for<'sub_context> Traversable<Visitor, CommitContext<'sub_context, SS, SM, E>>,
{
    fn for_each(&mut self, visitor: &mut Visitor, context: &mut CommitContext<'context, S, M, E>) {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut sub_messages,
            entry_point: context.entry_point,
        };
        self.target.for_each(visitor, &mut sub_context);
        context
            .messages
            .extend(sub_messages.into_iter().map(&self.lift_message));
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut CommitContext<'context, S, M, E>,
    ) -> bool {
        let mut sub_messages = Vec::new();
        let mut sub_context = CommitContext {
            id_stack: context.id_stack,
            state: (self.select_state)(context.state),
            messages: &mut sub_messages,
            entry_point: context.entry_point,
        };
        let result = self.target.for_id(id, visitor, &mut sub_context);
        context
            .messages
            .extend(sub_messages.into_iter().map(&self.lift_message));
        result
    }
}

impl<T, S, M, SS, SM> fmt::Debug for Adapt<T, S, M, SS, SM>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Adapt").field(&self.target).finish()
    }
}

fn with_sub_node<Callback, Output, V, CS, S, M, SS, SM, E>(
    node: &mut ViewNodeMut<Adapt<V, S, M, SS, SM>, Adapt<CS, S, M, SS, SM>, S, M, E>,
    callback: Callback,
) -> Output
where
    Callback: FnOnce(ViewNodeMut<V, CS, SS, SM, E>) -> Output,
    V: View<SS, SM, E>,
    CS: ComponentStack<SS, SM, E, View = V>,
{
    let select_state = &node.components.select_state;
    let lift_message = &node.components.lift_message;
    let mut sub_pending_view = node.pending_view.take().map(|view| view.target);
    let sub_node = ViewNodeMut {
        id: node.id,
        view: &mut node.view.target,
        pending_view: &mut sub_pending_view,
        view_state: node.view_state,
        children: &mut node.children.target,
        components: &mut node.components.target,
        dirty: &mut node.dirty,
    };
    let result = callback(sub_node);
    *node.pending_view =
        sub_pending_view.map(|view| Adapt::new(view, select_state.clone(), lift_message.clone()));
    result
}
