mod commit_subtree_visitor;
mod downward_event_visitor;
mod local_event_visitor;
mod update_subtree_visitor;
mod upward_event_visitor;

use std::any::Any;
use std::fmt;
use std::sync::Once;

use crate::component_stack::ComponentStack;
use crate::context::{IdContext, MessageContext, RenderContext};
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

pub struct ViewNode<V: View<S, M, R>, CS: ComponentStack<S, M, R, View = V>, S, M, R> {
    pub(crate) id: Id,
    pub(crate) state: Option<ViewNodeState<V, V::State>>,
    pub(crate) children: <V::Children as ElementSeq<S, M, R>>::Storage,
    pub(crate) components: CS,
    pub(crate) event_mask: &'static EventMask,
    pub(crate) dirty: bool,
}

impl<V, CS, S, M, R> ViewNode<V, CS, S, M, R>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    pub(crate) fn new(
        id: Id,
        view: V,
        children: <V::Children as ElementSeq<S, M, R>>::Storage,
        components: CS,
    ) -> Self {
        Self {
            id,
            state: Some(ViewNodeState::Uninitialized(view)),
            children,
            components,
            event_mask: <V::Children as ElementSeq<S, M, R>>::Storage::event_mask(),
            dirty: true,
        }
    }

    pub(crate) fn borrow_mut(&mut self) -> ViewNodeMut<V, CS, S, M, R> {
        ViewNodeMut {
            id: self.id,
            state: &mut self.state,
            children: &mut self.children,
            components: &mut self.components,
            dirty: &mut self.dirty,
        }
    }

    pub(crate) fn update_subtree(
        &mut self,
        id_tree: &IdTree<Depth>,
        store: &Store<S>,
        renderer: &mut R,
        context: &mut RenderContext,
    ) -> Vec<(IdPathBuf, Depth)> {
        let mut visitor = UpdateSubtreeVisitor::new(id_tree.root());
        visitor.visit(self, context, store, renderer)
    }

    pub(crate) fn commit_within(
        &mut self,
        mode: CommitMode,
        depth: Depth,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        if !self.dirty && !mode.is_propagatable() {
            return false;
        }

        context.push_id(self.id);

        let pre_result = match mode {
            CommitMode::Mount | CommitMode::Update => {
                self.children.commit(mode, context, store, renderer)
            }
            CommitMode::Unmount => {
                CS::commit(self.borrow_mut(), mode, depth, 0, context, store, renderer)
            }
        };

        let (result, node_state) = match (mode, self.state.take().unwrap()) {
            (CommitMode::Mount, ViewNodeState::Uninitialized(view)) => {
                let mut state = view.build(&mut self.children, store, renderer);
                view.lifecycle(
                    Lifecycle::Mount,
                    &mut state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                (true, ViewNodeState::Prepared(view, state))
            }
            (CommitMode::Mount, ViewNodeState::Prepared(view, mut state)) => {
                view.lifecycle(
                    Lifecycle::Remount,
                    &mut state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                (true, ViewNodeState::Prepared(view, state))
            }
            (CommitMode::Mount, ViewNodeState::Pending(view, pending_view, mut state)) => {
                view.lifecycle(
                    Lifecycle::Remount,
                    &mut state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                pending_view.lifecycle(
                    Lifecycle::Update(view),
                    &mut state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                (true, ViewNodeState::Prepared(pending_view, state))
            }
            (CommitMode::Update, ViewNodeState::Uninitialized(view)) => {
                (false, ViewNodeState::Uninitialized(view))
            }
            (CommitMode::Update, ViewNodeState::Prepared(view, state)) => {
                (false, ViewNodeState::Prepared(view, state))
            }
            (CommitMode::Update, ViewNodeState::Pending(view, pending_view, mut state)) => {
                pending_view.lifecycle(
                    Lifecycle::Update(view),
                    &mut state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                (true, ViewNodeState::Prepared(pending_view, state))
            }
            (CommitMode::Unmount, ViewNodeState::Uninitialized(view)) => {
                (false, ViewNodeState::Uninitialized(view))
            }
            (CommitMode::Unmount, ViewNodeState::Prepared(view, mut state)) => {
                view.lifecycle(
                    Lifecycle::Unmount,
                    &mut state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                (true, ViewNodeState::Prepared(view, state))
            }
            (CommitMode::Unmount, ViewNodeState::Pending(view, pending_view, mut state)) => {
                view.lifecycle(
                    Lifecycle::Unmount,
                    &mut state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                (true, ViewNodeState::Pending(view, pending_view, state))
            }
        };

        self.state = Some(node_state);
        self.dirty = false;

        let post_result = match mode {
            CommitMode::Mount | CommitMode::Update => {
                CS::commit(self.borrow_mut(), mode, depth, 0, context, store, renderer)
            }
            CommitMode::Unmount => self.children.commit(mode, context, store, renderer),
        };

        context.pop_id();

        pre_result | result | post_result
    }

    pub(crate) fn commit_subtree(
        &mut self,
        id_tree: &IdTree<Depth>,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        let mut visitor = CommitSubtreeVisitor::new(CommitMode::Update, id_tree.root());
        visitor.visit(self, context, store, renderer)
    }

    pub(crate) fn global_event(
        &mut self,
        event: &dyn Any,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        let mut visitor = DownwardEventVisitor::new(event, &[]);
        visitor.visit(self, context, store, renderer)
    }

    pub(crate) fn downward_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        let mut visitor = DownwardEventVisitor::new(event, id_path);
        visitor.visit(self, context, store, renderer)
    }

    pub(crate) fn upward_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        let mut visitor = UpwardEventVisitor::new(event, id_path);
        visitor.visit(self, context, store, renderer)
    }

    pub(crate) fn local_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        let mut visitor = LocalEventVisitor::new(event, id_path);
        visitor.visit(self, context, store, renderer)
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn state(&self) -> &ViewNodeState<V, V::State> {
        self.state.as_ref().unwrap()
    }

    pub fn state_mut(&mut self) -> &mut ViewNodeState<V, V::State> {
        self.state.as_mut().unwrap()
    }

    pub fn children(&self) -> &<V::Children as ElementSeq<S, M, R>>::Storage {
        &self.children
    }

    pub fn components(&self) -> &CS {
        &self.components
    }

    pub fn event_mask(&self) -> &EventMask {
        &self.event_mask
    }
}

#[derive(Debug)]
pub enum ViewNodeState<V, VS> {
    Uninitialized(V),
    Prepared(V, VS),
    Pending(V, V, VS),
}

impl<V, VS> ViewNodeState<V, VS> {
    pub fn map_view<F, W>(self, f: F) -> ViewNodeState<W, VS>
    where
        F: Fn(V) -> W,
    {
        match self {
            Self::Uninitialized(view) => ViewNodeState::Uninitialized(f(view)),
            Self::Prepared(view, state) => ViewNodeState::Prepared(f(view), state),
            Self::Pending(view, pending_view, state) => {
                ViewNodeState::Pending(f(view), f(pending_view), state)
            }
        }
    }

    pub fn extract(&self) -> (&V, Option<&VS>) {
        match self {
            ViewNodeState::Prepared(view, state) | ViewNodeState::Pending(view, _, state) => {
                (view, Some(state))
            }
            ViewNodeState::Uninitialized(view) => (view, None),
        }
    }

    pub fn extract_mut(&mut self) -> (&V, Option<&mut VS>) {
        match self {
            ViewNodeState::Prepared(view, state) | ViewNodeState::Pending(view, _, state) => {
                (view, Some(state))
            }
            ViewNodeState::Uninitialized(view) => (view, None),
        }
    }

    pub fn as_view(&self) -> &V {
        match self {
            Self::Prepared(view, _) | Self::Pending(view, _, _) | Self::Uninitialized(view) => view,
        }
    }

    pub fn as_view_state(&self) -> Option<&VS> {
        match self {
            ViewNodeState::Prepared(_, state) | ViewNodeState::Pending(_, _, state) => Some(state),
            ViewNodeState::Uninitialized(_) => None,
        }
    }
}

pub struct ViewNodeMut<'a, V: View<S, M, R>, CS: ?Sized, S, M, R> {
    pub(crate) id: Id,
    pub(crate) state: &'a mut Option<ViewNodeState<V, V::State>>,
    pub(crate) children: &'a mut <V::Children as ElementSeq<S, M, R>>::Storage,
    pub(crate) components: &'a mut CS,
    pub(crate) dirty: &'a mut bool,
}

impl<'a, V: View<S, M, R>, CS: ?Sized, S, M, R> ViewNodeMut<'a, V, CS, S, M, R> {
    pub(crate) fn as_view_ref(&self) -> ViewRef<'_, V, S, M, R> {
        ViewRef {
            state: self.state,
            children: self.children,
        }
    }
}

pub struct ViewRef<'a, V: View<S, M, R>, S, M, R> {
    state: &'a Option<ViewNodeState<V, V::State>>,
    children: &'a <V::Children as ElementSeq<S, M, R>>::Storage,
}

impl<'a, V: View<S, M, R>, S, M, R> ViewRef<'a, V, S, M, R> {
    pub fn view(&self) -> &V {
        self.state.as_ref().unwrap().as_view()
    }

    pub fn state(&self) -> &V::State {
        self.state.as_ref().unwrap().as_view_state().unwrap()
    }

    pub fn children(&self) -> &<V::Children as ElementSeq<S, M, R>>::Storage {
        &self.children
    }
}

pub trait ViewNodeSeq<S, M, R>:
    for<'a> Traversable<CommitSubtreeVisitor<'a>, MessageContext<M>, bool, S, M, R>
    + for<'a> Traversable<DownwardEventVisitor<'a>, MessageContext<M>, bool, S, M, R>
    + for<'a> Traversable<UpdateSubtreeVisitor<'a>, RenderContext, Vec<(IdPathBuf, Depth)>, S, M, R>
    + for<'a> Traversable<LocalEventVisitor<'a>, MessageContext<M>, bool, S, M, R>
    + for<'a> Traversable<UpwardEventVisitor<'a>, MessageContext<M>, bool, S, M, R>
{
    const SIZE_HINT: (usize, Option<usize>);

    const IS_STATIC: bool = {
        match Self::SIZE_HINT {
            (lower, Some(upper)) => lower == upper,
            _ => false,
        }
    };

    fn event_mask() -> &'static EventMask;

    fn len(&self) -> usize;

    fn id_range(&self) -> Option<(Id, Id)>;

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool;
}

impl<V, CS, S, M, R> ViewNodeSeq<S, M, R> for ViewNode<V, CS, S, M, R>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    const SIZE_HINT: (usize, Option<usize>) = (1, Some(1));

    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let children_mask = <V::Children as ElementSeq<S, M, R>>::Storage::event_mask();

            INIT.call_once(|| unsafe {
                if !children_mask.is_empty() {
                    EVENT_MASK.extend(children_mask);
                }
                let types = <V as EventListener>::Event::types().into_iter();
                EVENT_MASK.extend(types);
            });
        }

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        1
    }

    fn id_range(&self) -> Option<(Id, Id)> {
        Some((self.id, self.id))
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        self.commit_within(mode, 0, context, store, renderer)
    }
}

impl<'a, V, CS, S, M, R, Visitor> Traversable<Visitor, Visitor::Context, Visitor::Output, S, M, R>
    for ViewNode<V, CS, S, M, R>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
    Visitor: self::Visitor<Self, S, R>,
    Visitor::Context: IdContext,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut Visitor::Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Visitor::Output {
        context.push_id(self.id);
        let result = visitor.visit(self, context, store, renderer);
        context.pop_id();
        result
    }

    fn for_id(
        &mut self,
        id: Id,
        visitor: &mut Visitor,
        context: &mut Visitor::Context,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Option<Visitor::Output> {
        context.push_id(self.id);
        let result = if id == self.id {
            Some(visitor.visit(self, context, store, renderer))
        } else {
            None
        };
        context.pop_id();
        result
    }
}

impl<V, CS, S, M, R> fmt::Debug for ViewNode<V, CS, S, M, R>
where
    V: View<S, M, R> + fmt::Debug,
    V::State: fmt::Debug,
    <V::Children as ElementSeq<S, M, R>>::Storage: fmt::Debug,
    CS: ComponentStack<S, M, R, View = V> + fmt::Debug,
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
