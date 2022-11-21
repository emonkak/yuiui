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
use crate::store::Store;
use crate::view::View;

use broadcast_event_visitor::BroadcastEventVisitor;
use commit_subtree_visitor::CommitSubtreeVisitor;
use forward_event_visitor::ForwardEventVisitor;
use update_subtree_visitor::UpdateSubtreeVisitor;

pub struct ViewNode<V: View<S, M, B>, CS: ComponentStack<S, M, B, View = V>, S, M, B> {
    pub(crate) id: Id,
    pub(crate) depth: Depth,
    pub(crate) view: V,
    pub(crate) pending_view: Option<V>,
    pub(crate) state: Option<V::State>,
    pub(crate) children: <V::Children as ElementSeq<S, M, B>>::Storage,
    pub(crate) components: CS,
    pub(crate) dirty: bool,
}

impl<V, CS, S, M, B> ViewNode<V, CS, S, M, B>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    pub(crate) fn new(
        id: Id,
        view: V,
        children: <V::Children as ElementSeq<S, M, B>>::Storage,
        components: CS,
    ) -> Self {
        Self {
            id,
            depth: 0,
            view,
            pending_view: None,
            state: None,
            children,
            components,
            dirty: true,
        }
    }

    pub(crate) fn update_subtree(
        &mut self,
        id_tree: &IdTree<Depth>,
        store: &Store<S>,
        id_context: &mut IdContext,
    ) -> Vec<(IdPathBuf, Depth)> {
        let mut visitor = UpdateSubtreeVisitor::new(id_tree.root());
        let mut context = RenderContext { store };
        visitor.visit(self, &mut context, id_context);
        visitor.into_result()
    }

    pub(crate) fn commit_from(
        &mut self,
        mode: CommitMode,
        depth: Depth,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        backend: &B,
    ) -> bool {
        if !self.dirty && !mode.is_propagable() {
            return false;
        }

        id_context.push_id(self.id);

        let mut result = match mode {
            CommitMode::Mount | CommitMode::Update => self
                .children
                .commit(mode, id_context, store, messages, backend),
            CommitMode::Unmount => CS::commit(
                &mut self.into(),
                mode,
                depth,
                id_context,
                store,
                messages,
                backend,
            ),
        };

        result |= match (mode, self.pending_view.take(), self.state.as_mut()) {
            (CommitMode::Mount, None, None) => {
                let mut state = self.view.build(&mut self.children, store, backend);
                self.view.lifecycle(
                    Lifecycle::Mount,
                    &mut state,
                    &mut self.children,
                    id_context,
                    store,
                    messages,
                    backend,
                );
                self.state = Some(state);
                true
            }
            (CommitMode::Mount, Some(pending_view), None) => {
                let mut state = pending_view.build(&mut self.children, store, backend);
                pending_view.lifecycle(
                    Lifecycle::Mount,
                    &mut state,
                    &mut self.children,
                    id_context,
                    store,
                    messages,
                    backend,
                );
                self.state = Some(state);
                true
            }
            (CommitMode::Mount, None, Some(state)) => {
                self.view.lifecycle(
                    Lifecycle::Remount,
                    state,
                    &mut self.children,
                    id_context,
                    store,
                    messages,
                    backend,
                );
                true
            }
            (CommitMode::Mount, Some(pending_view), Some(state)) => {
                self.view.lifecycle(
                    Lifecycle::Remount,
                    state,
                    &mut self.children,
                    id_context,
                    store,
                    messages,
                    backend,
                );
                let old_view = mem::replace(&mut self.view, pending_view);
                self.view.lifecycle(
                    Lifecycle::Update(old_view),
                    state,
                    &mut self.children,
                    id_context,
                    store,
                    messages,
                    backend,
                );
                true
            }
            (CommitMode::Update, None, None) => false,
            (CommitMode::Update, None, Some(_)) => false,
            (CommitMode::Update, Some(_), None) => {
                unreachable!()
            }
            (CommitMode::Update, Some(pending_view), Some(state)) => {
                let old_view = mem::replace(&mut self.view, pending_view);
                self.view.lifecycle(
                    Lifecycle::Update(old_view),
                    state,
                    &mut self.children,
                    id_context,
                    store,
                    messages,
                    backend,
                );
                true
            }
            (CommitMode::Unmount, None, None) => false,
            (CommitMode::Unmount, None, Some(state)) => {
                self.view.lifecycle(
                    Lifecycle::Unmount,
                    state,
                    &mut self.children,
                    id_context,
                    store,
                    messages,
                    backend,
                );
                true
            }
            (CommitMode::Unmount, Some(pending_view), Some(state)) => {
                self.view.lifecycle(
                    Lifecycle::Unmount,
                    state,
                    &mut self.children,
                    id_context,
                    store,
                    messages,
                    backend,
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
                id_context,
                store,
                messages,
                backend,
            ),
            CommitMode::Unmount => self
                .children
                .commit(mode, id_context, store, messages, backend),
        };

        id_context.pop_id();

        result
    }

    pub(crate) fn commit_subtree(
        &mut self,
        id_tree: &IdTree<Depth>,
        id_context: &mut IdContext,
        store: &Store<S>,
        backend: &B,
    ) -> Vec<M> {
        let mut visitor = CommitSubtreeVisitor::new(CommitMode::Update, id_tree.root());
        let mut messages = Vec::new();
        let mut context = CommitContext {
            store,
            messages: &mut messages,
            backend,
        };
        visitor.visit(self, &mut context, id_context);
        messages
    }

    pub(crate) fn forward_event(
        &mut self,
        payload: &dyn Any,
        destination: &IdPath,
        id_context: &mut IdContext,
        store: &Store<S>,
        backend: &B,
    ) -> Vec<M> {
        let mut visitor = ForwardEventVisitor::new(payload, destination);
        let mut messages = Vec::new();
        let mut context = CommitContext {
            store,
            messages: &mut messages,
            backend,
        };
        visitor.visit(self, &mut context, id_context);
        messages
    }

    pub(crate) fn broadcast_event(
        &mut self,
        payload: &dyn Any,
        destinations: &[IdPathBuf],
        id_context: &mut IdContext,
        store: &Store<S>,
        backend: &B,
    ) -> Vec<M> {
        let id_tree = IdTree::from_iter(destinations);
        let cursor = id_tree.root();
        let mut visitor = BroadcastEventVisitor::new(payload, cursor);
        let mut messages = Vec::new();
        let mut context = CommitContext {
            store,
            messages: &mut messages,
            backend,
        };
        visitor.visit(self, &mut context, id_context);
        messages
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn depth(&self) -> Depth {
        self.depth
    }

    pub fn view(&self) -> &V {
        &self.view
    }

    pub fn state(&self) -> Option<&V::State> {
        self.state.as_ref()
    }

    pub fn view_mut(&mut self) -> &mut V {
        &mut self.view
    }

    pub fn state_mut(&mut self) -> Option<&mut V::State> {
        self.state.as_mut()
    }

    pub fn children(&self) -> &<V::Children as ElementSeq<S, M, B>>::Storage {
        &self.children
    }

    pub fn components(&self) -> &CS {
        &self.components
    }
}

impl<V, CS, S, M, B> fmt::Debug for ViewNode<V, CS, S, M, B>
where
    V: View<S, M, B> + fmt::Debug,
    V::State: fmt::Debug,
    <V::Children as ElementSeq<S, M, B>>::Storage: fmt::Debug,
    CS: ComponentStack<S, M, B, View = V> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ViewNode")
            .field("id", &self.id)
            .field("view", &self.view)
            .field("pending_view", &self.pending_view)
            .field("state", &self.state)
            .field("children", &self.children)
            .field("components", &self.components)
            .field("dirty", &self.dirty)
            .finish()
    }
}

pub struct ViewNodeMut<'a, V: View<S, M, B>, CS: ?Sized, S, M, B> {
    pub(crate) id: Id,
    pub(crate) depth: Depth,
    pub(crate) view: &'a mut V,
    pub(crate) pending_view: &'a mut Option<V>,
    pub(crate) state: &'a mut Option<V::State>,
    pub(crate) children: &'a mut <V::Children as ElementSeq<S, M, B>>::Storage,
    pub(crate) components: &'a mut CS,
    pub(crate) dirty: &'a mut bool,
}

impl<'a, V, CS, S, M, B> ViewNodeMut<'a, V, CS, S, M, B>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn depth(&self) -> Depth {
        self.depth
    }

    pub fn view(&mut self) -> &mut V {
        self.view
    }

    pub fn state(&mut self) -> &mut V::State {
        self.state.as_mut().unwrap()
    }

    pub fn children(&mut self) -> &mut <V::Children as ElementSeq<S, M, B>>::Storage {
        self.children
    }
}

impl<'a, V, CS, S, M, B> From<&'a mut ViewNode<V, CS, S, M, B>> for ViewNodeMut<'a, V, CS, S, M, B>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    fn from(node: &'a mut ViewNode<V, CS, S, M, B>) -> Self {
        Self {
            id: node.id,
            depth: node.depth,
            view: &mut node.view,
            pending_view: &mut node.pending_view,
            state: &mut node.state,
            children: &mut node.children,
            components: &mut node.components,
            dirty: &mut node.dirty,
        }
    }
}

pub trait ViewNodeSeq<S, M, B>:
    for<'a, 'context> Traversable<
        BroadcastEventVisitor<'a>,
        CommitContext<'context, S, M, B>,
        S,
        M,
        B,
    > + for<'a, 'context> Traversable<
        CommitSubtreeVisitor<'a>,
        CommitContext<'context, S, M, B>,
        S,
        M,
        B,
    > + for<'a, 'context> Traversable<
        ForwardEventVisitor<'a>,
        CommitContext<'context, S, M, B>,
        S,
        M,
        B,
    > + for<'a, 'context> Traversable<UpdateSubtreeVisitor<'a>, RenderContext<'context, S>, S, M, B>
{
    const SIZE_HINT: (usize, Option<usize>);

    const IS_STATIC: bool = {
        match Self::SIZE_HINT {
            (lower, Some(upper)) => lower == upper,
            _ => false,
        }
    };

    fn len(&self) -> usize;

    fn id_range(&self) -> Option<(Id, Id)>;

    fn commit(
        &mut self,
        mode: CommitMode,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        backend: &B,
    ) -> bool;

    fn gc(&mut self);
}

impl<V, CS, S, M, B> ViewNodeSeq<S, M, B> for ViewNode<V, CS, S, M, B>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    const SIZE_HINT: (usize, Option<usize>) = (1, Some(1));

    fn len(&self) -> usize {
        1
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        Some((self.id, self.id))
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        backend: &B,
    ) -> bool {
        self.commit_from(mode, 0, id_context, store, messages, backend)
    }

    fn gc(&mut self) {
        if !<V::Children as ElementSeq<S, M, B>>::Storage::IS_STATIC {
            self.children.gc();
        }
    }
}

pub trait Traversable<Visitor, Context, S, M, B> {
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

pub trait Visitor<Node, Context, S, M, B> {
    fn visit(&mut self, node: &mut Node, context: &mut Context, id_context: &mut IdContext);
}

impl<'a, V, CS, S, M, B, Visitor, Context> Traversable<Visitor, Context, S, M, B>
    for ViewNode<V, CS, S, M, B>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
    Visitor: self::Visitor<Self, Context, S, M, B>,
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
    pub store: &'context Store<S>,
}

#[derive(Debug)]
pub struct CommitContext<'context, S, M, B> {
    pub store: &'context Store<S>,
    pub messages: &'context mut Vec<M>,
    pub backend: &'context B,
}
