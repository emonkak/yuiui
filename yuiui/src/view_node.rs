mod commit_subtree_visitor;
mod downward_event_visitor;
mod local_event_visitor;
mod update_subtree_visitor;
mod upward_event_visitor;

use std::any::Any;
use std::fmt;
use std::sync::Once;

use crate::component_stack::ComponentStack;
use crate::context::{MessageContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::Lifecycle;
use crate::event::{Event, EventListener, EventMask};
use crate::id::{Depth, Id, IdPath, IdPathBuf, IdTree};
use crate::state::Store;
use crate::traversable::{Traversable, Visitor};
use crate::view::View;

use commit_subtree_visitor::CommitSubtreeVisitor;
use downward_event_visitor::DownwardEventVisitor;
use local_event_visitor::LocalEventVisitor;
use update_subtree_visitor::UpdateSubtreeVisitor;
use upward_event_visitor::UpwardEventVisitor;

pub struct ViewNode<V: View<S, M, B>, CS: ComponentStack<S, M, B, View = V>, S, M, B> {
    pub(crate) id: Id,
    pub(crate) state: Option<ViewNodeState<V, V::State>>,
    pub(crate) children: <V::Children as ElementSeq<S, M, B>>::Storage,
    pub(crate) components: CS,
    pub(crate) event_mask: &'static EventMask,
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
            state: Some(ViewNodeState::Uninitialized(view)),
            children,
            components,
            event_mask: <V::Children as ElementSeq<S, M, B>>::Storage::event_mask(),
            dirty: true,
        }
    }

    pub(crate) fn borrow_mut(&mut self) -> ViewNodeMut<V, CS, S, M, B> {
        ViewNodeMut {
            id: self.id,
            state: &mut self.state,
            children: &mut self.children,
            components: &mut self.components,
            dirty: &mut self.dirty,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn node_state(&self) -> &ViewNodeState<V, V::State> {
        self.state.as_ref().unwrap()
    }

    pub fn as_view_state(&self) -> Option<&V::State> {
        match self.state.as_ref().unwrap() {
            ViewNodeState::Prepared(_, view_state) | ViewNodeState::Pending(_, _, view_state) => {
                Some(view_state)
            }
            ViewNodeState::Uninitialized(_) => None,
        }
    }

    pub fn children(&self) -> &<V::Children as ElementSeq<S, M, B>>::Storage {
        &self.children
    }

    pub fn components(&self) -> &CS {
        &self.components
    }

    pub fn event_mask(&self) -> &EventMask {
        &self.event_mask
    }

    pub fn update_subtree(
        &mut self,
        id_tree: &IdTree<Depth>,
        store: &Store<S>,
        backend: &mut B,
        context: &mut RenderContext,
    ) -> Vec<(IdPathBuf, Depth)> {
        let mut visitor = UpdateSubtreeVisitor::new(id_tree.root());
        visitor.visit(self, context, store, backend)
    }

    pub fn commit_within(
        &mut self,
        mode: CommitMode,
        depth: Depth,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        if !self.dirty && !mode.is_propagatable() {
            return false;
        }

        context.begin_id(self.id);

        let pre_result = match mode {
            CommitMode::Mount | CommitMode::Update => {
                self.children.commit(mode, context, store, backend)
            }
            CommitMode::Unmount => self
                .components
                .commit(mode, depth, 0, context, store, backend),
        };

        let (result, node_state) = match (mode, self.state.take().unwrap()) {
            (CommitMode::Mount, ViewNodeState::Uninitialized(view)) => {
                let mut view_state = view.build(&self.children, store, backend);
                view.lifecycle(
                    Lifecycle::Mount,
                    &mut view_state,
                    &self.children,
                    context,
                    store,
                    backend,
                );
                (true, ViewNodeState::Prepared(view, view_state))
            }
            (CommitMode::Mount, ViewNodeState::Prepared(view, mut view_state)) => {
                view.lifecycle(
                    Lifecycle::Mount,
                    &mut view_state,
                    &self.children,
                    context,
                    store,
                    backend,
                );
                (true, ViewNodeState::Prepared(view, view_state))
            }
            (CommitMode::Mount, ViewNodeState::Pending(view, pending_view, mut view_state)) => {
                view.lifecycle(
                    Lifecycle::Mount,
                    &mut view_state,
                    &self.children,
                    context,
                    store,
                    backend,
                );
                pending_view.lifecycle(
                    Lifecycle::Update(view),
                    &mut view_state,
                    &self.children,
                    context,
                    store,
                    backend,
                );
                (true, ViewNodeState::Prepared(pending_view, view_state))
            }
            (CommitMode::Update, ViewNodeState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Update, ViewNodeState::Prepared(view, view_state)) => {
                (false, ViewNodeState::Prepared(view, view_state))
            }
            (CommitMode::Update, ViewNodeState::Pending(view, pending_view, mut view_state)) => {
                pending_view.lifecycle(
                    Lifecycle::Update(view),
                    &mut view_state,
                    &self.children,
                    context,
                    store,
                    backend,
                );
                (true, ViewNodeState::Prepared(pending_view, view_state))
            }
            (CommitMode::Unmount, ViewNodeState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Unmount, ViewNodeState::Prepared(view, mut view_state)) => {
                view.lifecycle(
                    Lifecycle::Unmount,
                    &mut view_state,
                    &self.children,
                    context,
                    store,
                    backend,
                );
                (true, ViewNodeState::Prepared(view, view_state))
            }
            (CommitMode::Unmount, ViewNodeState::Pending(view, pending_view, mut view_state)) => {
                view.lifecycle(
                    Lifecycle::Unmount,
                    &mut view_state,
                    &self.children,
                    context,
                    store,
                    backend,
                );
                (true, ViewNodeState::Pending(view, pending_view, view_state))
            }
        };

        self.state = Some(node_state);
        self.dirty = false;

        let post_result = match mode {
            CommitMode::Mount | CommitMode::Update => self
                .components
                .commit(mode, depth, 0, context, store, backend),
            CommitMode::Unmount => self.children.commit(mode, context, store, backend),
        };

        context.end_id();

        pre_result | result | post_result
    }

    pub fn commit_subtree(
        &mut self,
        id_tree: &IdTree<Depth>,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let mut visitor = CommitSubtreeVisitor::new(CommitMode::Update, id_tree.root());
        visitor.visit(self, context, store, backend)
    }

    pub fn global_event(
        &mut self,
        event: &dyn Any,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let mut visitor = DownwardEventVisitor::new(event);
        visitor.visit(self, context, store, backend)
    }

    pub fn downward_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let mut visitor = DownwardEventVisitor::new(event);
        self.for_id_path(id_path, &mut visitor, context, store, backend)
            .unwrap_or_default()
    }

    pub fn upward_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let mut visitor = UpwardEventVisitor::new(event, id_path);
        visitor.visit(self, context, store, backend)
    }

    pub fn local_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let mut visitor = LocalEventVisitor::new(event);
        self.for_id_path(id_path, &mut visitor, context, store, backend)
            .unwrap_or(false)
    }
}

pub struct ViewNodeMut<'a, V: View<S, M, B>, CS: ?Sized, S, M, B> {
    pub(crate) id: Id,
    pub(crate) state: &'a mut Option<ViewNodeState<V, V::State>>,
    pub(crate) children: &'a mut <V::Children as ElementSeq<S, M, B>>::Storage,
    pub(crate) components: &'a mut CS,
    pub(crate) dirty: &'a mut bool,
}

pub trait ViewNodeSeq<S, M, B>:
    for<'a> Traversable<CommitSubtreeVisitor<'a>, MessageContext<M>, bool, S, B>
    + for<'a> Traversable<DownwardEventVisitor<'a>, MessageContext<M>, bool, S, B>
    + for<'a> Traversable<UpdateSubtreeVisitor<'a>, RenderContext, Vec<(IdPathBuf, Depth)>, S, B>
    + for<'a> Traversable<LocalEventVisitor<'a>, MessageContext<M>, bool, S, B>
    + for<'a> Traversable<UpwardEventVisitor<'a>, MessageContext<M>, bool, S, B>
{
    fn event_mask() -> &'static EventMask;

    fn len(&self) -> usize;

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool;
}

impl<V, CS, S, M, B> ViewNodeSeq<S, M, B> for ViewNode<V, CS, S, M, B>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let children_mask = <V::Children as ElementSeq<S, M, B>>::Storage::event_mask();

            INIT.call_once(|| unsafe {
                if !children_mask.is_empty() {
                    EVENT_MASK.extend(children_mask);
                }
                let mut types = Vec::new();
                <V as EventListener>::Event::collect_types(&mut types);
                if !types.is_empty() {
                    EVENT_MASK.extend(types);
                }
            });
        }

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        1
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        self.commit_within(mode, 0, context, store, backend)
    }
}

impl<'a, V, CS, S, M, B, Visitor> Traversable<Visitor, RenderContext, Visitor::Output, S, B>
    for ViewNode<V, CS, S, M, B>
where
    V: View<S, M, B>,
    <V::Children as ElementSeq<S, M, B>>::Storage:
        Traversable<Visitor, RenderContext, Visitor::Output, S, B>,
    CS: ComponentStack<S, M, B, View = V>,
    Visitor: self::Visitor<Self, S, B, Context = RenderContext>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &mut B,
    ) -> Visitor::Output {
        context.begin_id(self.id);
        let result = visitor.visit(self, context, store, backend);
        context.end_id();
        result
    }

    fn for_id_path(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &mut B,
    ) -> Option<Visitor::Output> {
        context.begin_id(self.id);
        let result = if self.id == Id::from_bottom(id_path) {
            Some(visitor.visit(self, context, store, backend))
        } else if self.id == Id::from_top(id_path) {
            debug_assert!(id_path.len() > 0);
            self.children
                .for_id_path(&id_path[1..], visitor, context, store, backend)
        } else {
            None
        };
        context.end_id();
        result
    }
}

impl<'a, V, CS, S, M, B, Visitor> Traversable<Visitor, MessageContext<M>, Visitor::Output, S, B>
    for ViewNode<V, CS, S, M, B>
where
    V: View<S, M, B>,
    <V::Children as ElementSeq<S, M, B>>::Storage:
        Traversable<Visitor, MessageContext<M>, Visitor::Output, S, B>,
    CS: ComponentStack<S, M, B, View = V>,
    Visitor: self::Visitor<Self, S, B, Context = MessageContext<M>>,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> Visitor::Output {
        context.begin_id(self.id);
        let result = visitor.visit(self, context, store, backend);
        context.end_id();
        result
    }

    fn for_id_path(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> Option<Visitor::Output> {
        context.begin_id(self.id);
        let result = if self.id == Id::from_top(id_path) {
            Some(visitor.visit(self, context, store, backend))
        } else if self.id == Id::from_bottom(id_path) {
            debug_assert!(id_path.len() > 0);
            let id_path = &id_path[1..];
            self.children
                .for_id_path(id_path, visitor, context, store, backend)
        } else {
            None
        };
        context.end_id();
        result
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
            .field("state", &self.state)
            .field("children", &self.children)
            .field("components", &self.components)
            .field("event_mask", &self.event_mask)
            .field("dirty", &self.dirty)
            .finish()
    }
}

#[derive(Debug)]
pub enum ViewNodeState<T, S> {
    Uninitialized(T),
    Prepared(T, S),
    Pending(T, T, S),
}

impl<T, S> ViewNodeState<T, S> {
    pub fn map_view<F, U>(self, f: F) -> ViewNodeState<U, S>
    where
        F: Fn(T) -> U,
    {
        match self {
            Self::Uninitialized(view) => ViewNodeState::Uninitialized(f(view)),
            Self::Prepared(view, view_state) => ViewNodeState::Prepared(f(view), view_state),
            Self::Pending(view, pending_view, view_state) => {
                ViewNodeState::Pending(f(view), f(pending_view), view_state)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitMode {
    Mount,
    Unmount,
    Update,
}

impl CommitMode {
    pub fn is_propagatable(&self) -> bool {
        match self {
            Self::Mount | Self::Unmount => true,
            Self::Update => false,
        }
    }
}
