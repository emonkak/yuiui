mod broadcast_event_visitor;
mod commit_subtree_visitor;
mod forward_event_visitor;
mod update_subtree_visitor;

use std::any::Any;
use std::{fmt, mem};

use crate::component_stack::ComponentStack;
use crate::element::ElementSeq;
use crate::event::Lifecycle;
use crate::id::{Depth, Id, IdContext, IdPath, IdPathBuf, IdTree};
use crate::view::View;

use broadcast_event_visitor::BroadcastEventVisitor;
use commit_subtree_visitor::CommitSubtreeVisitor;
use forward_event_visitor::ForwardEventVisitor;
use update_subtree_visitor::UpdateSubtreeVisitor;

pub struct ViewNode<V: View<S, M, E>, CS: ComponentStack<S, M, E, View = V>, S, M, E> {
    pub(crate) id: Id,
    pub(crate) view: V,
    pub(crate) pending_view: Option<V>,
    pub(crate) view_state: Option<V::State>,
    pub(crate) children: <V::Children as ElementSeq<S, M, E>>::Storage,
    pub(crate) components: CS,
    pub(crate) dirty: bool,
}

impl<V, CS, S, M, E> ViewNode<V, CS, S, M, E>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    pub(crate) fn new(
        id: Id,
        view: V,
        children: <V::Children as ElementSeq<S, M, E>>::Storage,
        components: CS,
    ) -> Self {
        Self {
            id,
            view,
            pending_view: None,
            view_state: None,
            children,
            components,
            dirty: true,
        }
    }

    pub(crate) fn update_subtree(
        &mut self,
        id_tree: &IdTree<Depth>,
        state: &S,
        id_context: &mut IdContext,
    ) -> Vec<(IdPathBuf, Depth)> {
        let mut visitor = UpdateSubtreeVisitor::new(id_tree.root());
        let mut context = RenderContext { state };
        visitor.visit(self, &mut context, id_context);
        visitor.into_result()
    }

    pub(crate) fn commit_from(
        &mut self,
        mode: CommitMode,
        depth: Depth,
        state: &S,
        id_context: &mut IdContext,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool {
        if !self.dirty && !mode.is_propagable() {
            return false;
        }

        id_context.push_id(self.id);

        let mut result = match mode {
            CommitMode::Mount | CommitMode::Update => {
                self.children
                    .commit(mode, state, id_context, messages, entry_point)
            }
            CommitMode::Unmount => CS::commit(
                &mut self.into(),
                mode,
                depth,
                state,
                messages,
                entry_point,
                id_context,
            ),
        };

        result |= match (mode, self.pending_view.take(), self.view_state.as_mut()) {
            (CommitMode::Mount, None, None) => {
                let mut view_state = self.view.build(&mut self.children, state, entry_point);
                self.view.lifecycle(
                    Lifecycle::Mount,
                    &mut view_state,
                    &mut self.children,
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                self.view_state = Some(view_state);
                true
            }
            (CommitMode::Mount, Some(pending_view), None) => {
                let mut view_state = pending_view.build(&mut self.children, state, entry_point);
                pending_view.lifecycle(
                    Lifecycle::Mount,
                    &mut view_state,
                    &mut self.children,
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                self.view_state = Some(view_state);
                true
            }
            (CommitMode::Mount, None, Some(view_state)) => {
                self.view.lifecycle(
                    Lifecycle::Remount,
                    view_state,
                    &mut self.children,
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                true
            }
            (CommitMode::Mount, Some(pending_view), Some(view_state)) => {
                self.view.lifecycle(
                    Lifecycle::Remount,
                    view_state,
                    &mut self.children,
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                let old_view = mem::replace(&mut self.view, pending_view);
                self.view.lifecycle(
                    Lifecycle::Update(old_view),
                    view_state,
                    &mut self.children,
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                true
            }
            (CommitMode::Update, None, None) => false,
            (CommitMode::Update, None, Some(_)) => false,
            (CommitMode::Update, Some(_), None) => {
                unreachable!()
            }
            (CommitMode::Update, Some(pending_view), Some(view_state)) => {
                let old_view = mem::replace(&mut self.view, pending_view);
                self.view.lifecycle(
                    Lifecycle::Update(old_view),
                    view_state,
                    &mut self.children,
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                true
            }
            (CommitMode::Unmount, None, None) => false,
            (CommitMode::Unmount, None, Some(view_state)) => {
                self.view.lifecycle(
                    Lifecycle::Unmount,
                    view_state,
                    &mut self.children,
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                true
            }
            (CommitMode::Unmount, Some(pending_view), Some(view_state)) => {
                self.view.lifecycle(
                    Lifecycle::Unmount,
                    view_state,
                    &mut self.children,
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                self.pending_view = Some(pending_view);
                true
            }
            (CommitMode::Unmount, Some(_), None) => {
                unreachable!()
            }
        };

        self.dirty = false;

        result |= match mode {
            CommitMode::Mount | CommitMode::Update => CS::commit(
                &mut self.into(),
                mode,
                depth,
                state,
                messages,
                entry_point,
                id_context,
            ),
            CommitMode::Unmount => {
                self.children
                    .commit(mode, state, id_context, messages, entry_point)
            }
        };

        id_context.pop_id();

        result
    }

    pub(crate) fn commit_subtree(
        &mut self,
        id_tree: &IdTree<Depth>,
        id_context: &mut IdContext,
        state: &S,
        entry_point: &E,
    ) -> Vec<M> {
        let mut visitor = CommitSubtreeVisitor::new(CommitMode::Update, id_tree.root());
        let mut messages = Vec::new();
        let mut context = CommitContext {
            state,
            messages: &mut messages,
            entry_point,
        };
        visitor.visit(self, &mut context, id_context);
        messages
    }

    pub(crate) fn forward_event(
        &mut self,
        payload: &dyn Any,
        destination: &IdPath,
        id_context: &mut IdContext,
        state: &S,
        entry_point: &E,
    ) -> Vec<M> {
        let mut visitor = ForwardEventVisitor::new(payload, destination);
        let mut messages = Vec::new();
        let mut context = CommitContext {
            state,
            messages: &mut messages,
            entry_point,
        };
        visitor.visit(self, &mut context, id_context);
        messages
    }

    pub(crate) fn broadcast_event(
        &mut self,
        payload: &dyn Any,
        destinations: &[IdPathBuf],
        id_context: &mut IdContext,
        state: &S,
        entry_point: &E,
    ) -> Vec<M> {
        let id_tree = IdTree::from_iter(destinations);
        let cursor = id_tree.root();
        let mut visitor = BroadcastEventVisitor::new(payload, cursor);
        let mut messages = Vec::new();
        let mut context = CommitContext {
            state,
            messages: &mut messages,
            entry_point,
        };
        visitor.visit(self, &mut context, id_context);
        messages
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn view(&self) -> &V {
        &self.view
    }

    pub fn view_state(&self) -> Option<&V::State> {
        self.view_state.as_ref()
    }

    pub fn view_mut(&mut self) -> &mut V {
        &mut self.view
    }

    pub fn view_state_mut(&mut self) -> Option<&mut V::State> {
        self.view_state.as_mut()
    }

    pub fn children(&self) -> &<V::Children as ElementSeq<S, M, E>>::Storage {
        &self.children
    }

    pub fn components(&self) -> &CS {
        &self.components
    }
}

impl<V, CS, S, M, E> fmt::Debug for ViewNode<V, CS, S, M, E>
where
    V: View<S, M, E> + fmt::Debug,
    V::State: fmt::Debug,
    <V::Children as ElementSeq<S, M, E>>::Storage: fmt::Debug,
    CS: ComponentStack<S, M, E, View = V> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ViewNode")
            .field("id", &self.id)
            .field("view", &self.view)
            .field("pending_view", &self.pending_view)
            .field("view_state", &self.view_state)
            .field("children", &self.children)
            .field("components", &self.components)
            .field("dirty", &self.dirty)
            .finish()
    }
}

pub struct ViewNodeMut<'a, V: View<S, M, E>, CS: ?Sized, S, M, E> {
    pub(crate) id: Id,
    pub(crate) view: &'a mut V,
    pub(crate) pending_view: &'a mut Option<V>,
    pub(crate) view_state: &'a mut Option<V::State>,
    pub(crate) children: &'a mut <V::Children as ElementSeq<S, M, E>>::Storage,
    pub(crate) components: &'a mut CS,
    pub(crate) dirty: &'a mut bool,
}

impl<'a, V, CS, S, M, E> ViewNodeMut<'a, V, CS, S, M, E>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn view(&mut self) -> &mut V {
        self.view
    }

    pub fn view_state(&mut self) -> &mut V::State {
        self.view_state.as_mut().unwrap()
    }

    pub fn children(&mut self) -> &mut <V::Children as ElementSeq<S, M, E>>::Storage {
        self.children
    }
}

impl<'a, V, CS, S, M, E> From<&'a mut ViewNode<V, CS, S, M, E>> for ViewNodeMut<'a, V, CS, S, M, E>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    fn from(node: &'a mut ViewNode<V, CS, S, M, E>) -> Self {
        Self {
            id: node.id,
            view: &mut node.view,
            pending_view: &mut node.pending_view,
            view_state: &mut node.view_state,
            children: &mut node.children,
            components: &mut node.components,
            dirty: &mut node.dirty,
        }
    }
}

pub trait ViewNodeSeq<S, M, E>:
    for<'a, 'context> Traversable<
        BroadcastEventVisitor<'a>,
        CommitContext<'context, S, M, E>,
        S,
        M,
        E,
    > + for<'a, 'context> Traversable<
        CommitSubtreeVisitor<'a>,
        CommitContext<'context, S, M, E>,
        S,
        M,
        E,
    > + for<'a, 'context> Traversable<
        ForwardEventVisitor<'a>,
        CommitContext<'context, S, M, E>,
        S,
        M,
        E,
    > + for<'a, 'context> Traversable<UpdateSubtreeVisitor<'a>, RenderContext<'context, S>, S, M, E>
{
    const SIZE_HINT: (usize, Option<usize>);

    const IS_STATIC: bool = {
        match Self::SIZE_HINT {
            (lower, Some(upper)) => lower == upper,
            _ => false,
        }
    };

    fn len(&self) -> usize;

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        id_context: &mut IdContext,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool;

    fn gc(&mut self);
}

impl<V, CS, S, M, E> ViewNodeSeq<S, M, E> for ViewNode<V, CS, S, M, E>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    const SIZE_HINT: (usize, Option<usize>) = (1, Some(1));

    fn len(&self) -> usize {
        1
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        id_context: &mut IdContext,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool {
        self.commit_from(mode, 0, state, id_context, messages, entry_point)
    }

    fn gc(&mut self) {
        if !<V::Children as ElementSeq<S, M, E>>::Storage::IS_STATIC {
            self.children.gc();
        }
    }
}

pub trait Traversable<Visitor, Context, S, M, E> {
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    );

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    ) -> bool;
}

pub trait Visitor<Node, Context, S, M, E> {
    fn visit(&mut self, node: &mut Node, context: &mut Context, id_context: &mut IdContext);
}

impl<'a, V, CS, S, M, E, Visitor, Context> Traversable<Visitor, Context, S, M, E>
    for ViewNode<V, CS, S, M, E>
where
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
    Visitor: self::Visitor<Self, Context, S, M, E>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    ) {
        id_context.push_id(self.id);
        let result = visitor.visit(self, context, id_context);
        id_context.pop_id();
        result
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Context,
        id_context: &mut IdContext,
    ) -> bool {
        id_context.push_id(self.id);
        let result = if id == self.id {
            visitor.visit(self, context, id_context);
            true
        } else {
            false
        };
        id_context.pop_id();
        result
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitMode {
    Mount,
    Unmount,
    Update,
}

impl CommitMode {
    pub fn is_propagable(&self) -> bool {
        match self {
            Self::Mount | Self::Unmount => true,
            Self::Update => false,
        }
    }
}

#[derive(Debug)]
pub struct RenderContext<'context, S> {
    pub state: &'context S,
}

#[derive(Debug)]
pub struct CommitContext<'context, S, M, E> {
    pub state: &'context S,
    pub messages: &'context mut Vec<M>,
    pub entry_point: &'context E,
}
