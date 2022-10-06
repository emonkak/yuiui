mod commit_subtree_visitor;
mod downward_event_visitor;
mod local_event_visitor;
mod update_subtree_visitor;
mod upward_event_visitor;

use std::any::Any;
use std::fmt;
use std::mem;
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
    pub(crate) view: V,
    pub(crate) pending_view: Option<V>,
    pub(crate) state: Option<V::State>,
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
            view,
            pending_view: None,
            state: None,
            children,
            components,
            event_mask: <Self as ViewNodeSeq<S, M, R>>::event_mask(),
            dirty: true,
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

        let mut result = match mode {
            CommitMode::Mount | CommitMode::Update => {
                self.children.commit(mode, context, store, renderer)
            }
            CommitMode::Unmount => {
                CS::commit(self.into(), mode, depth, 0, context, store, renderer)
            }
        };

        result |= match (mode, self.pending_view.take(), self.state.as_mut()) {
            (CommitMode::Mount, None, None) => {
                let mut state = self.view.build(&mut self.children, store, renderer);
                self.view.lifecycle(
                    Lifecycle::Mount,
                    &mut state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                self.state = Some(state);
                true
            }
            (CommitMode::Mount, Some(pending_view), None) => {
                let mut state = pending_view.build(&mut self.children, store, renderer);
                pending_view.lifecycle(
                    Lifecycle::Mount,
                    &mut state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                self.state = Some(state);
                true
            }
            (CommitMode::Mount, None, Some(state)) => {
                self.view.lifecycle(
                    Lifecycle::Remount,
                    state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                true
            }
            (CommitMode::Mount, Some(pending_view), Some(state)) => {
                self.view.lifecycle(
                    Lifecycle::Remount,
                    state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                let old_view = mem::replace(&mut self.view, pending_view);
                self.view.lifecycle(
                    Lifecycle::Update(old_view),
                    state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
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
                    context,
                    store,
                    renderer,
                );
                true
            }
            (CommitMode::Unmount, None, None) => false,
            (CommitMode::Unmount, None, Some(state)) => {
                self.view.lifecycle(
                    Lifecycle::Unmount,
                    state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
                );
                true
            }
            (CommitMode::Unmount, Some(pending_view), Some(state)) => {
                self.view.lifecycle(
                    Lifecycle::Unmount,
                    state,
                    &mut self.children,
                    context,
                    store,
                    renderer,
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
            CommitMode::Mount | CommitMode::Update => {
                CS::commit(self.into(), mode, depth, 0, context, store, renderer)
            }
            CommitMode::Unmount => self.children.commit(mode, context, store, renderer),
        };

        context.pop_id();

        result
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

pub struct ViewNodeMut<'a, V: View<S, M, R>, CS: ?Sized, S, M, R> {
    pub(crate) id: Id,
    pub(crate) view: &'a mut V,
    pub(crate) pending_view: &'a mut Option<V>,
    pub(crate) state: &'a mut Option<V::State>,
    pub(crate) children: &'a mut <V::Children as ElementSeq<S, M, R>>::Storage,
    pub(crate) components: &'a mut CS,
    pub(crate) dirty: &'a mut bool,
}

impl<'a, V, CS, S, M, R> From<&'a mut ViewNode<V, CS, S, M, R>> for ViewNodeMut<'a, V, CS, S, M, R>
where
    V: View<S, M, R>,
    CS: ComponentStack<S, M, R, View = V>,
{
    fn from(node: &'a mut ViewNode<V, CS, S, M, R>) -> Self {
        Self {
            id: node.id,
            view: &mut node.view,
            pending_view: &mut node.pending_view,
            state: &mut node.state,
            children: &mut node.children,
            components: &mut node.components,
            dirty: &mut node.dirty,
        }
    }
}

pub struct ViewNodeRef<'a, V: View<S, M, R>, S, M, R> {
    view: &'a V,
    state: &'a Option<V::State>,
    children: &'a <V::Children as ElementSeq<S, M, R>>::Storage,
}

impl<'a, V: View<S, M, R>, CS: ?Sized, S, M, R> From<ViewNodeMut<'a, V, CS, S, M, R>>
    for ViewNodeRef<'a, V, S, M, R>
{
    fn from(node: ViewNodeMut<'a, V, CS, S, M, R>) -> Self {
        Self {
            view: node.view,
            state: node.state,
            children: node.children,
        }
    }
}

impl<'a, V: View<S, M, R>, S, M, R> ViewNodeRef<'a, V, S, M, R> {
    pub fn view(&self) -> &V {
        self.view
    }

    pub fn state(&self) -> &V::State {
        self.state.as_ref().unwrap()
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

    fn gc(&mut self);
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
                EVENT_MASK.extend(children_mask);
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

    fn gc(&mut self) {
        if Self::SIZE_HINT.1.is_none() {
            self.children.gc();
        }
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
            .field("view", &self.view)
            .field("pending_view", &self.pending_view)
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
